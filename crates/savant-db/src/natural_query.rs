//! Natural language query parser for LLM database integration
//! 
//! Provides intent classification, entity extraction, and query building for natural language queries

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc, NaiveDate};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use sqlx::SqlitePool;
use tokio::sync::RwLock;
use uuid;

/// Intent types for natural language queries
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IntentType {
    FindConversations,
    AnalyzeSpeaker,
    SearchContent,
    GetStatistics,
    ExportData,
    ListSpeakers,
    GetTopics,
    Unknown,
}

impl std::fmt::Display for IntentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            IntentType::FindConversations => write!(f, "find_conversations"),
            IntentType::AnalyzeSpeaker => write!(f, "analyze_speaker"),
            IntentType::SearchContent => write!(f, "search_content"),
            IntentType::GetStatistics => write!(f, "get_statistics"),
            IntentType::ExportData => write!(f, "export_data"),
            IntentType::ListSpeakers => write!(f, "list_speakers"),
            IntentType::GetTopics => write!(f, "get_topics"),
            IntentType::Unknown => write!(f, "unknown"),
        }
    }
}

/// Extracted intent from natural language query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueryIntent {
    pub intent_type: IntentType,
    pub entities: HashMap<String, String>,
    pub confidence: f32,
    pub original_query: String,
}

/// LLM trait for query processing
#[async_trait::async_trait]
pub trait LLMClient: Send + Sync {
    async fn complete(&self, prompt: &str) -> Result<String>;
}

/// Conversation context for follow-up queries
#[derive(Debug, Clone)]
pub struct ConversationContext {
    pub session_id: String,
    pub previous_queries: Vec<String>,
    pub active_filters: HashMap<String, String>,
    pub last_results: Vec<String>, // IDs of last returned results
}

/// Context manager for maintaining conversation state
pub struct ConversationContextManager {
    contexts: Arc<RwLock<HashMap<String, ConversationContext>>>,
}

/// Query complexity levels for rate limiting
#[derive(Debug, Clone, PartialEq)]
pub enum QueryComplexity {
    Low,
    Medium, 
    High,
}

/// LLM-powered query processor (replacing regex-based approach)
pub struct QueryProcessor {
    llm_client: Option<Box<dyn LLMClient>>,
    simple_patterns: Vec<(Regex, String)>,
    context_manager: Arc<ConversationContextManager>,
    pool: SqlitePool,
}

/// Enhanced query result with LLM processing
#[derive(Debug, Serialize, Deserialize)]
pub struct LLMQueryResult {
    pub intent: String,
    pub sql_query: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub confidence: f32,
}

/// Legacy natural language query parser (for backward compatibility)
pub struct NaturalLanguageQueryParser {
    intent_classifier: IntentClassifier,
    entity_extractor: EntityExtractor,
    query_builder: QueryBuilder,
    pool: SqlitePool,
}

/// Intent classification using pattern matching
pub struct IntentClassifier {
    patterns: HashMap<IntentType, Vec<Regex>>,
}

/// Entity extraction for speakers, dates, topics
pub struct EntityExtractor {
    speaker_patterns: Vec<Regex>,
    date_patterns: Vec<Regex>,
    topic_patterns: Vec<Regex>,
    number_patterns: Regex,
}

/// Query builder for converting intents to SQL
pub struct QueryBuilder {
    templates: HashMap<IntentType, String>,
}

/// Query execution result
#[derive(Debug, Serialize, Deserialize)]
pub struct QueryResult {
    pub intent: QueryIntent,
    pub sql_query: String,
    pub results: serde_json::Value,
    pub execution_time_ms: u64,
    pub result_count: usize,
}

impl ConversationContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn enhance_query(&self, session_id: &str, query: &str) -> String {
        let contexts = self.contexts.read().await;
        if let Some(context) = contexts.get(session_id) {
            // Add context to help LLM understand follow-up queries
            format!(
                "Previous queries: {:?}\nActive filters: {:?}\nCurrent query: {}",
                context.previous_queries.iter().rev().take(3).collect::<Vec<_>>(),
                context.active_filters,
                query
            )
        } else {
            query.to_string()
        }
    }
    
    pub async fn update_context(&self, session_id: &str, query: &str, results: &[String]) {
        let mut contexts = self.contexts.write().await;
        let context = contexts.entry(session_id.to_string()).or_insert_with(|| {
            ConversationContext {
                session_id: session_id.to_string(),
                previous_queries: Vec::new(),
                active_filters: HashMap::new(),
                last_results: Vec::new(),
            }
        });
        
        context.previous_queries.push(query.to_string());
        context.last_results = results.to_vec();
        
        // Keep only last 10 queries
        if context.previous_queries.len() > 10 {
            context.previous_queries = context.previous_queries.split_off(context.previous_queries.len() - 10);
        }
    }
}

impl QueryProcessor {
    pub fn new(pool: SqlitePool, llm_client: Option<Box<dyn LLMClient>>) -> Self {
        let simple_patterns = vec![
            (Regex::new(r"(?i)\b(statistics|stats|summary|overview|total)\b").unwrap(), "get_statistics".to_string()),
            (Regex::new(r"(?i)\b(list|show)\s+(speakers?|people)").unwrap(), "list_speakers".to_string()),
        ];
        
        Self {
            llm_client,
            simple_patterns,
            context_manager: Arc::new(ConversationContextManager::new()),
            pool,
        }
    }
    
    pub async fn process_query(&self, natural_query: &str, session_id: &str) -> Result<LLMQueryResult> {
        // First try simple pattern matching for common queries
        if let Some(simple_result) = self.try_simple_patterns(natural_query) {
            return Ok(simple_result);
        }
        
        // Use LLM for complex query understanding if available
        if let Some(ref llm_client) = self.llm_client {
            let enhanced_query = self.context_manager.enhance_query(session_id, natural_query).await;
            
            let llm_response = llm_client.complete(&format!(
                "Convert this natural language query to a structured database query:
                Query: {}
                
                Return JSON with:
                - intent: find_conversations|analyze_speaker|search_content|get_statistics|list_speakers
                - sql_query: parameterized SQL query with ? placeholders
                - parameters: object with parameter values
                - confidence: 0.0-1.0
                
                JSON:", enhanced_query
            )).await?;
            
            return serde_json::from_str(&llm_response).map_err(Into::into);
        }
        
        // Fallback to pattern-based processing
        self.fallback_processing(natural_query)
    }
    
    fn try_simple_patterns(&self, query: &str) -> Option<LLMQueryResult> {
        for (pattern, intent) in &self.simple_patterns {
            if pattern.is_match(query) {
                let sql_query = match intent.as_str() {
                    "get_statistics" => "SELECT COUNT(DISTINCT c.id) as total_conversations, COUNT(DISTINCT s.speaker) as unique_speakers, COUNT(s.id) as total_segments FROM conversations c LEFT JOIN segments s ON c.id = s.conversation_id".to_string(),
                    "list_speakers" => "SELECT speaker, COUNT(DISTINCT conversation_id) as conversation_count FROM segments WHERE speaker IS NOT NULL GROUP BY speaker ORDER BY conversation_count DESC LIMIT 20".to_string(),
                    _ => continue,
                };
                
                return Some(LLMQueryResult {
                    intent: intent.clone(),
                    sql_query,
                    parameters: HashMap::new(),
                    confidence: 0.9,
                });
            }
        }
        None
    }
    
    fn fallback_processing(&self, query: &str) -> Result<LLMQueryResult> {
        // Simple fallback to content search
        let sql_query = "SELECT s.text, s.speaker, s.timestamp, c.title as conversation_title FROM segments s JOIN conversations c ON s.conversation_id = c.id WHERE s.text LIKE ? OR s.processed_text LIKE ? ORDER BY s.timestamp DESC LIMIT 50".to_string();
        
        let search_term = format!("%{}%", query);
        let mut parameters = HashMap::new();
        parameters.insert("0".to_string(), serde_json::Value::String(search_term.clone()));
        parameters.insert("1".to_string(), serde_json::Value::String(search_term));
        
        Ok(LLMQueryResult {
            intent: "search_content".to_string(),
            sql_query,
            parameters,
            confidence: 0.5,
        })
    }
}

impl NaturalLanguageQueryParser {
    /// Create a new parser instance
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            intent_classifier: IntentClassifier::new(),
            entity_extractor: EntityExtractor::new(),
            query_builder: QueryBuilder::new(),
            pool,
        }
    }
    
    /// Parse natural language query and extract intent
    pub async fn parse_query(&self, query: &str) -> Result<QueryIntent> {
        // Classify intent
        let mut intent = self.intent_classifier.classify(query);
        
        // Extract entities
        let entities = self.entity_extractor.extract_entities(query);
        intent.entities.extend(entities);
        
        // Normalize speaker names using database lookup
        if let Some(speaker_input) = intent.entities.get("speaker") {
            if let Ok(normalized) = self.normalize_speaker_name(speaker_input).await {
                intent.entities.insert("speaker".to_string(), normalized);
            }
        }
        
        Ok(intent)
    }
    
    /// Build SQL query from intent
    pub fn build_sql_query(&self, intent: &QueryIntent) -> Result<String> {
        self.query_builder.build_query(intent)
    }
    
    /// Execute a natural language query end-to-end
    pub async fn execute_natural_query(&self, query: &str) -> Result<QueryResult> {
        let start_time = std::time::Instant::now();
        
        // Parse intent
        let intent = self.parse_query(query).await?;
        
        // Build SQL
        let sql_query = self.build_sql_query(&intent)?;
        
        // Execute query
        let results = self.execute_sql_query(&sql_query, &intent).await?;
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        let result_count = match &results {
            serde_json::Value::Array(arr) => arr.len(),
            serde_json::Value::Object(_) => 1,
            _ => 0,
        };
        
        Ok(QueryResult {
            intent,
            sql_query,
            results,
            execution_time_ms: execution_time,
            result_count,
        })
    }
    
    /// Execute SQL query with proper type handling
    async fn execute_sql_query(&self, sql: &str, intent: &QueryIntent) -> Result<serde_json::Value> {
        match intent.intent_type {
            IntentType::FindConversations => {
                let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
                let conversations: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                    serde_json::json!({
                        "id": row.get::<String, _>("id"),
                        "title": row.get::<Option<String>, _>("title"),
                        "start_time": row.get::<DateTime<Utc>, _>("start_time"),
                        "participants": row.get::<Option<String>, _>("participants")
                            .map(|s| s.split(',').map(|p| p.trim()).collect::<Vec<_>>())
                            .unwrap_or_default(),
                        "segment_count": row.get::<i64, _>("segment_count"),
                        "total_duration": row.get::<Option<f64>, _>("total_duration").unwrap_or(0.0),
                    })
                }).collect();
                Ok(serde_json::Value::Array(conversations))
            }
            
            IntentType::AnalyzeSpeaker => {
                let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
                let stats: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                    serde_json::json!({
                        "speaker": row.get::<String, _>("speaker"),
                        "conversation_count": row.get::<i64, _>("conversation_count"),
                        "total_duration": row.get::<f64, _>("total_duration"),
                        "total_segments": row.get::<i64, _>("total_segments"),
                        "avg_confidence": row.get::<f64, _>("avg_confidence"),
                    })
                }).collect();
                Ok(serde_json::Value::Array(stats))
            }
            
            IntentType::SearchContent => {
                let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
                let results: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                    serde_json::json!({
                        "text": row.get::<String, _>("text"),
                        "speaker": row.get::<String, _>("speaker"),
                        "timestamp": row.get::<DateTime<Utc>, _>("timestamp"),
                        "conversation_title": row.get::<Option<String>, _>("conversation_title"),
                        "confidence": row.get::<Option<f64>, _>("confidence"),
                    })
                }).collect();
                Ok(serde_json::Value::Array(results))
            }
            
            IntentType::GetStatistics => {
                let row = sqlx::query(sql).fetch_one(&self.pool).await?;
                Ok(serde_json::json!({
                    "total_conversations": row.get::<i64, _>("total_conversations"),
                    "unique_speakers": row.get::<i64, _>("unique_speakers"),
                    "total_segments": row.get::<i64, _>("total_segments"),
                    "total_duration": row.get::<Option<f64>, _>("total_duration").unwrap_or(0.0),
                }))
            }
            
            IntentType::ListSpeakers => {
                let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
                let speakers: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                    serde_json::json!({
                        "name": row.get::<String, _>("speaker"),
                        "conversation_count": row.get::<i64, _>("conversation_count"),
                        "total_duration": row.get::<f64, _>("total_duration"),
                    })
                }).collect();
                Ok(serde_json::Value::Array(speakers))
            }
            
            _ => {
                // Generic fallback
                let rows = sqlx::query(sql).fetch_all(&self.pool).await?;
                let results: Vec<serde_json::Value> = rows.into_iter().map(|row| {
                    let mut obj = serde_json::Map::new();
                    for column in row.columns() {
                        let column_name = column.name();
                        let value = match column.type_info().name() {
                            "TEXT" => serde_json::Value::String(
                                row.get::<Option<String>, _>(column_name).unwrap_or_default()
                            ),
                            "INTEGER" => serde_json::Value::Number(
                                serde_json::Number::from(row.get::<i64, _>(column_name))
                            ),
                            "REAL" => serde_json::Value::Number(
                                serde_json::Number::from_f64(row.get::<f64, _>(column_name))
                                    .unwrap_or(serde_json::Number::from(0))
                            ),
                            _ => serde_json::Value::Null,
                        };
                        obj.insert(column_name.to_string(), value);
                    }
                    serde_json::Value::Object(obj)
                }).collect();
                Ok(serde_json::Value::Array(results))
            }
        }
    }
    
    /// Normalize speaker name using fuzzy matching
    async fn normalize_speaker_name(&self, input: &str) -> Result<String> {
        // Get all known speakers
        let rows = sqlx::query("SELECT DISTINCT speaker FROM segments WHERE speaker IS NOT NULL")
            .fetch_all(&self.pool)
            .await?;
        
        let speakers: Vec<String> = rows.into_iter()
            .map(|row| row.get::<String, _>("speaker"))
            .collect();
        
        // Simple fuzzy matching - find closest speaker name
        let input_lower = input.to_lowercase();
        
        // Exact match first
        for speaker in &speakers {
            if speaker.to_lowercase() == input_lower {
                return Ok(speaker.clone());
            }
        }
        
        // Partial match
        for speaker in &speakers {
            if speaker.to_lowercase().contains(&input_lower) || 
               input_lower.contains(&speaker.to_lowercase()) {
                return Ok(speaker.clone());
            }
        }
        
        // Return original if no match found
        Ok(input.to_string())
    }
}

impl IntentClassifier {
    pub fn new() -> Self {
        let mut patterns = HashMap::new();
        
        // Find conversations patterns
        patterns.insert(IntentType::FindConversations, vec![
            Regex::new(r"(?i)\b(find|show|list|get)\b.*\b(conversations?|talks?|meetings?|chats?)\b").unwrap(),
            Regex::new(r"(?i)\b(conversations?|talks?|meetings?)\b.*\b(with|between|involving)\b").unwrap(),
            Regex::new(r"(?i)\b(who|what)\b.*\b(talked|spoke|said)\b").unwrap(),
        ]);
        
        // Speaker analysis patterns
        patterns.insert(IntentType::AnalyzeSpeaker, vec![
            Regex::new(r"(?i)\b(analyze|statistics|stats|info|details)\b.*\b(speaker|person)\b").unwrap(),
            Regex::new(r"(?i)\bhow\s+(much|long|many)\b.*\b(talked|spoke|conversation)\b").unwrap(),
            Regex::new(r"(?i)\b(speaker|person)\b.*\b(statistics|stats|analysis)\b").unwrap(),
        ]);
        
        // Content search patterns
        patterns.insert(IntentType::SearchContent, vec![
            Regex::new(r"(?i)\b(search|find|look)\b.*\b(for|about|mentioning)\b").unwrap(),
            Regex::new(r"(?i)\b(mentions?|contains?|includes?)\b").unwrap(),
            Regex::new(r"(?i)\b(conversations?|talks?)\b.*\b(about|regarding|concerning)\b").unwrap(),
        ]);
        
        // Statistics patterns
        patterns.insert(IntentType::GetStatistics, vec![
            Regex::new(r"(?i)\b(statistics|stats|summary|overview|total)\b").unwrap(),
            Regex::new(r"(?i)\bhow\s+(many|much)\b.*\b(conversations?|recordings?|speakers?)\b").unwrap(),
            Regex::new(r"(?i)\b(database|system)\b.*\b(stats|statistics|summary)\b").unwrap(),
        ]);
        
        // List speakers patterns
        patterns.insert(IntentType::ListSpeakers, vec![
            Regex::new(r"(?i)\b(list|show|get)\b.*\b(speakers?|people|participants)\b").unwrap(),
            Regex::new(r"(?i)\b(who|speakers?)\b.*\b(available|recorded|database)\b").unwrap(),
        ]);
        
        // Export patterns
        patterns.insert(IntentType::ExportData, vec![
            Regex::new(r"(?i)\b(export|download|save|extract)\b").unwrap(),
            Regex::new(r"(?i)\bget\s+data\b").unwrap(),
        ]);
        
        Self { patterns }
    }
    
    pub fn classify(&self, query: &str) -> QueryIntent {
        for (intent_type, regexes) in &self.patterns {
            for regex in regexes {
                if regex.is_match(query) {
                    return QueryIntent {
                        intent_type: intent_type.clone(),
                        entities: HashMap::new(),
                        confidence: 0.8, // Pattern-based confidence
                        original_query: query.to_string(),
                    };
                }
            }
        }
        
        // Default fallback to content search
        QueryIntent {
            intent_type: IntentType::SearchContent,
            entities: HashMap::new(),
            confidence: 0.3,
            original_query: query.to_string(),
        }
    }
}

impl EntityExtractor {
    pub fn new() -> Self {
        Self {
            speaker_patterns: vec![
                Regex::new(r"(?i)\b(?:with|from|by|speaker|person)\s+([a-zA-Z][a-zA-Z0-9_\s]+?)(?:\s|$|,|\.)").unwrap(),
                Regex::new(r"(?i)\b([a-zA-Z][a-zA-Z0-9_]+)\s+(?:said|talked|spoke)").unwrap(),
            ],
            date_patterns: vec![
                Regex::new(r"(?i)\b(yesterday|today|tomorrow)\b").unwrap(),
                Regex::new(r"(?i)\blast\s+(week|month|year|day)\b").unwrap(),
                Regex::new(r"(?i)\bthis\s+(week|month|year|day)\b").unwrap(),
                Regex::new(r"(?i)\b(\d{4}-\d{2}-\d{2})\b").unwrap(),
                Regex::new(r"(?i)\b(january|february|march|april|may|june|july|august|september|october|november|december)\b").unwrap(),
            ],
            topic_patterns: vec![
                Regex::new(r"(?i)\babout\s+([a-zA-Z][a-zA-Z0-9_\s]+?)(?:\s|$|,|\.)").unwrap(),
                Regex::new(r"(?i)\bregarding\s+([a-zA-Z][a-zA-Z0-9_\s]+?)(?:\s|$|,|\.)").unwrap(),
                Regex::new(r"(?i)\bconcerning\s+([a-zA-Z][a-zA-Z0-9_\s]+?)(?:\s|$|,|\.)").unwrap(),
            ],
            number_patterns: Regex::new(r"(?i)\b(\d+)\s*(?:conversations?|results?|items?)\b").unwrap(),
        }
    }
    
    pub fn extract_entities(&self, query: &str) -> HashMap<String, String> {
        let mut entities = HashMap::new();
        
        // Extract speaker names
        for pattern in &self.speaker_patterns {
            if let Some(captures) = pattern.captures(query) {
                if let Some(speaker) = captures.get(1) {
                    let speaker_name = speaker.as_str().trim().to_lowercase();
                    if !speaker_name.is_empty() && speaker_name.len() > 1 {
                        entities.insert("speaker".to_string(), speaker_name);
                        break;
                    }
                }
            }
        }
        
        // Extract date references
        for pattern in &self.date_patterns {
            if let Some(captures) = pattern.captures(query) {
                if let Some(date) = captures.get(1) {
                    entities.insert("date".to_string(), date.as_str().to_string());
                    break;
                }
            }
        }
        
        // Extract topics
        for pattern in &self.topic_patterns {
            if let Some(captures) = pattern.captures(query) {
                if let Some(topic) = captures.get(1) {
                    let topic_text = topic.as_str().trim();
                    if !topic_text.is_empty() {
                        entities.insert("topic".to_string(), topic_text.to_string());
                        break;
                    }
                }
            }
        }
        
        // Extract numbers for limits
        if let Some(captures) = self.number_patterns.captures(query) {
            if let Some(number) = captures.get(1) {
                entities.insert("limit".to_string(), number.as_str().to_string());
            }
        }
        
        entities
    }
}

/// Query feedback for learning and optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserFeedback {
    Good,
    BadResults,
    TooSlow,
    WrongIntent,
    Irrelevant,
}

/// Query optimizer for learning from user feedback
pub struct QueryOptimizer {
    feedback_history: Arc<RwLock<HashMap<String, Vec<(UserFeedback, chrono::DateTime<Utc>)>>>>,
    successful_patterns: Arc<RwLock<HashMap<String, String>>>, // query pattern -> successful SQL
    pool: SqlitePool,
}

/// Query error with user-friendly suggestions
#[derive(Debug, Serialize, Deserialize)]
pub enum QueryError {
    Ambiguous { 
        query: String, 
        suggestions: Vec<String> 
    },
    NoResults { 
        query: String, 
        did_you_mean: Option<String> 
    },
    TooManyResults { 
        count: usize, 
        suggestion: String 
    },
    ExecutionError {
        query: String,
        error_message: String,
    },
}

impl QueryError {
    pub fn to_user_message(&self) -> String {
        match self {
            Self::Ambiguous { suggestions, .. } => {
                format!("Your query was ambiguous. Did you mean:\n{}", 
                    suggestions.join("\n• "))
            }
            Self::NoResults { query, did_you_mean } => {
                let base = format!("No results found for '{}'.", query);
                if let Some(suggestion) = did_you_mean {
                    format!("{} Did you mean '{}'?", base, suggestion)
                } else {
                    format!("{} Try a different search term or check spelling.", base)
                }
            }
            Self::TooManyResults { count, suggestion } => {
                format!("Found {} results (showing first 50). {}", count, suggestion)
            }
            Self::ExecutionError { query, error_message } => {
                format!("Error executing query '{}': {}", query, error_message)
            }
        }
    }
}

impl QueryOptimizer {
    pub fn new(pool: SqlitePool) -> Self {
        Self {
            feedback_history: Arc::new(RwLock::new(HashMap::new())),
            successful_patterns: Arc::new(RwLock::new(HashMap::new())),
            pool,
        }
    }
    
    pub async fn learn_from_feedback(
        &self, 
        query: &str, 
        sql_query: &str,
        results: &serde_json::Value, 
        feedback: UserFeedback
    ) -> Result<()> {
        let mut history = self.feedback_history.write().await;
        let query_key = self.normalize_query_for_learning(query);
        
        let feedback_entry = (feedback.clone(), chrono::Utc::now());
        history.entry(query_key.clone()).or_insert_with(Vec::new).push(feedback_entry);
        
        match feedback {
            UserFeedback::Good => {
                // Cache this successful pattern
                let mut patterns = self.successful_patterns.write().await;
                patterns.insert(query_key, sql_query.to_string());
                
                // Store in database for persistence
                self.store_successful_pattern(query, sql_query).await?;
            }
            UserFeedback::BadResults | UserFeedback::WrongIntent => {
                // Log for analysis and potential prompt adjustment
                self.log_failed_query(query, sql_query, &feedback).await?;
            }
            _ => {}
        }
        
        Ok(())
    }
    
    pub async fn get_query_suggestions(&self, query: &str) -> Vec<String> {
        let normalized = self.normalize_query_for_learning(query);
        let patterns = self.successful_patterns.read().await;
        
        // Find similar successful queries
        let mut suggestions = Vec::new();
        for (pattern, _) in patterns.iter() {
            if self.calculate_similarity(&normalized, pattern) > 0.7 {
                suggestions.push(pattern.clone());
            }
        }
        
        suggestions.truncate(3); // Limit to top 3 suggestions
        suggestions
    }
    
    async fn store_successful_pattern(&self, query: &str, sql_query: &str) -> Result<()> {
        let query_id = uuid::Uuid::new_v4().to_string();
        
        sqlx::query(
            "INSERT OR REPLACE INTO query_history 
             (id, natural_query, structured_query, intent_type, success, timestamp) 
             VALUES (?, ?, ?, 'learned_pattern', true, ?)")
            .bind(&query_id)
            .bind(query)
            .bind(sql_query)
            .bind(chrono::Utc::now())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    async fn log_failed_query(&self, query: &str, sql_query: &str, feedback: &UserFeedback) -> Result<()> {
        let query_id = uuid::Uuid::new_v4().to_string();
        let error_message = format!("User feedback: {:?}", feedback);
        
        sqlx::query(
            "INSERT INTO query_history 
             (id, natural_query, structured_query, intent_type, success, error_message, timestamp) 
             VALUES (?, ?, ?, 'failed_pattern', false, ?, ?)")
            .bind(&query_id)
            .bind(query)
            .bind(sql_query)
            .bind(&error_message)
            .bind(chrono::Utc::now())
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    fn normalize_query_for_learning(&self, query: &str) -> String {
        // Normalize query for pattern matching
        query.to_lowercase()
            .split_whitespace()
            .filter(|word| ![
                "the", "a", "an", "and", "or", "but", "in", "on", "at", "to", "for", "of", "with", "by"
            ].contains(word))
            .collect::<Vec<_>>()
            .join(" ")
    }
    
    fn calculate_similarity(&self, query1: &str, query2: &str) -> f32 {
        // Simple word overlap similarity
        let words1: HashSet<&str> = query1.split_whitespace().collect();
        let words2: HashSet<&str> = query2.split_whitespace().collect();
        
        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f32 / union as f32
        }
    }
}

impl QueryBuilder {
    pub fn new() -> Self {
        let mut templates = HashMap::new();
        
        templates.insert(
            IntentType::FindConversations,
            r#"SELECT c.id, c.title, c.start_time, c.end_time, c.context,
                      COUNT(s.id) as segment_count,
                      SUM(s.end_time - s.start_time) as total_duration,
                      GROUP_CONCAT(DISTINCT s.speaker) as participants
               FROM conversations c
               LEFT JOIN segments s ON c.id = s.conversation_id
               {where_clause}
               GROUP BY c.id
               ORDER BY c.start_time DESC
               LIMIT {limit}"#.to_string(),
        );
        
        templates.insert(
            IntentType::AnalyzeSpeaker,
            r#"SELECT speaker,
                      COUNT(DISTINCT conversation_id) as conversation_count,
                      SUM(end_time - start_time) as total_duration,
                      COUNT(*) as total_segments,
                      AVG(COALESCE(confidence, 0.0)) as avg_confidence
               FROM segments
               WHERE speaker = '{speaker}'
               GROUP BY speaker"#.to_string(),
        );
        
        templates.insert(
            IntentType::SearchContent,
            r#"SELECT s.text, s.speaker, s.timestamp, s.confidence,
                      c.title as conversation_title
               FROM segments s
               JOIN conversations c ON s.conversation_id = c.id
               WHERE s.text LIKE '%{search_term}%'
                  OR s.processed_text LIKE '%{search_term}%'
               ORDER BY s.timestamp DESC
               LIMIT {limit}"#.to_string(),
        );
        
        templates.insert(
            IntentType::GetStatistics,
            r#"SELECT COUNT(DISTINCT c.id) as total_conversations,
                      COUNT(DISTINCT s.speaker) as unique_speakers,
                      COUNT(s.id) as total_segments,
                      SUM(s.end_time - s.start_time) as total_duration
               FROM conversations c
               LEFT JOIN segments s ON c.id = s.conversation_id"#.to_string(),
        );
        
        templates.insert(
            IntentType::ListSpeakers,
            r#"SELECT speaker,
                      COUNT(DISTINCT conversation_id) as conversation_count,
                      SUM(end_time - start_time) as total_duration
               FROM segments
               WHERE speaker IS NOT NULL AND speaker != ''
               GROUP BY speaker
               ORDER BY total_duration DESC
               LIMIT {limit}"#.to_string(),
        );
        
        Self { templates }
    }
    
    pub fn build_query(&self, intent: &QueryIntent) -> Result<String> {
        let template = self.templates.get(&intent.intent_type)
            .ok_or_else(|| anyhow!("Unknown intent type: {:?}", intent.intent_type))?;
        
        let mut query = template.clone();
        
        // Replace placeholders with extracted entities
        let limit = intent.entities.get("limit")
            .and_then(|s| s.parse::<usize>().ok())
            .unwrap_or(50);
        query = query.replace("{limit}", &limit.to_string());
        
        if let Some(speaker) = intent.entities.get("speaker") {
            query = query.replace("{speaker}", speaker);
        }
        
        if let Some(topic) = intent.entities.get("topic") {
            query = query.replace("{search_term}", topic);
        } else {
            // For content search without explicit topic, use the whole query
            let search_term = intent.original_query.split_whitespace()
                .filter(|word| !["find", "search", "show", "get", "about", "for"].contains(&word.to_lowercase().as_str()))
                .collect::<Vec<_>>()
                .join(" ");
            query = query.replace("{search_term}", &search_term);
        }
        
        // Handle WHERE clauses for FindConversations
        if intent.intent_type == IntentType::FindConversations {
            let mut where_conditions = Vec::new();
            
            if let Some(speaker) = intent.entities.get("speaker") {
                where_conditions.push(format!("s.speaker = '{}'", speaker));
            }
            
            if let Some(date) = intent.entities.get("date") {
                // Simple date handling - could be enhanced
                match date.as_str() {
                    "today" => where_conditions.push("DATE(c.start_time) = DATE('now')".to_string()),
                    "yesterday" => where_conditions.push("DATE(c.start_time) = DATE('now', '-1 day')".to_string()),
                    "last week" => where_conditions.push("c.start_time >= datetime('now', '-7 days')".to_string()),
                    _ => {
                        if date.len() == 10 && date.contains('-') {
                            where_conditions.push(format!("DATE(c.start_time) = '{}'", date));
                        }
                    }
                }
            }
            
            let where_clause = if where_conditions.is_empty() {
                "WHERE 1=1".to_string()
            } else {
                format!("WHERE {}", where_conditions.join(" AND "))
            };
            
            query = query.replace("{where_clause}", &where_clause);
        }
        
        Ok(query)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_intent_classification() {
        let classifier = IntentClassifier::new();
        
        let intent = classifier.classify("Find all conversations with John");
        assert_eq!(intent.intent_type, IntentType::FindConversations);
        
        let intent = classifier.classify("Show me speaker statistics");
        assert_eq!(intent.intent_type, IntentType::AnalyzeSpeaker);
        
        let intent = classifier.classify("Search for project alpha");
        assert_eq!(intent.intent_type, IntentType::SearchContent);
    }
    
    #[test]
    fn test_entity_extraction() {
        let extractor = EntityExtractor::new();
        
        let entities = extractor.extract_entities("Find conversations with Alice from last week");
        assert!(entities.contains_key("speaker"));
        assert!(entities.contains_key("date"));
        
        let entities = extractor.extract_entities("Search for project alpha discussions");
        assert!(entities.contains_key("topic"));
    }
    
    #[test]
    fn test_query_building() {
        let builder = QueryBuilder::new();
        
        let intent = QueryIntent {
            intent_type: IntentType::AnalyzeSpeaker,
            entities: {
                let mut map = HashMap::new();
                map.insert("speaker".to_string(), "john".to_string());
                map
            },
            confidence: 0.8,
            original_query: "Analyze speaker john".to_string(),
        };
        
        let query = builder.build_query(&intent).unwrap();
        assert!(query.contains("WHERE speaker = 'john'"));
    }
}