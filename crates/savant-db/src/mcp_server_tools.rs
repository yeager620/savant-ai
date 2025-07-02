//! MCP server tool implementations with enhanced query processing
//! 
//! Contains the tool handlers for natural language database queries

use anyhow::{anyhow, Result};
use serde_json::{json, Value};
use std::time::Instant;
use sqlx::Row;

use crate::mcp_server::MCPServer;
use crate::natural_query::UserFeedback;

impl MCPServer {
    /// Handle list tools request
    pub async fn handle_list_tools(&self) -> Result<Value> {
        let tools = vec![
            json!({
                "name": "query_conversations",
                "description": "Query conversations using natural language with LLM-powered intent understanding",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Natural language query about conversations"
                        },
                        "session_id": {
                            "type": "string",
                            "description": "Session ID for context management (optional)"
                        }
                    },
                    "required": ["query"]
                }
            }),
            json!({
                "name": "get_speaker_analytics",
                "description": "Get detailed analytics and statistics for specific speakers",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "speaker": {
                            "type": "string",
                            "description": "Speaker name to analyze"
                        },
                        "include_interactions": {
                            "type": "boolean",
                            "description": "Include speaker interaction analysis",
                            "default": false
                        }
                    },
                    "required": ["speaker"]
                }
            }),
            json!({
                "name": "search_semantic",
                "description": "Perform semantic search across conversation content with full-text search",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "term": {
                            "type": "string",
                            "description": "Search term or phrase"
                        },
                        "limit": {
                            "type": "integer",
                            "description": "Maximum number of results",
                            "default": 20
                        },
                        "min_confidence": {
                            "type": "number",
                            "description": "Minimum confidence score for results",
                            "default": 0.0
                        }
                    },
                    "required": ["term"]
                }
            }),
            json!({
                "name": "get_conversation_context",
                "description": "Retrieve detailed context and segments for specific conversations",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "conversation_id": {
                            "type": "string",
                            "description": "Conversation ID to retrieve"
                        },
                        "include_segments": {
                            "type": "boolean",
                            "description": "Include all conversation segments",
                            "default": true
                        }
                    },
                    "required": ["conversation_id"]
                }
            }),
            json!({
                "name": "list_speakers",
                "description": "List all speakers with activity statistics and interaction matrices",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "include_stats": {
                            "type": "boolean",
                            "description": "Include detailed statistics for each speaker",
                            "default": true
                        },
                        "sort_by": {
                            "type": "string",
                            "enum": ["activity", "alphabetical", "recent"],
                            "description": "Sort order for speakers",
                            "default": "activity"
                        }
                    }
                }
            }),
            json!({
                "name": "learn_from_feedback",
                "description": "Provide feedback to improve query understanding and results",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Original natural language query"
                        },
                        "sql_query": {
                            "type": "string",
                            "description": "Generated SQL query"
                        },
                        "feedback": {
                            "type": "string",
                            "enum": ["Good", "BadResults", "TooSlow", "WrongIntent", "Irrelevant"],
                            "description": "User feedback on query results"
                        },
                        "results": {
                            "type": "object",
                            "description": "Query results for learning context"
                        }
                    },
                    "required": ["query", "feedback"]
                }
            }),
            json!({
                "name": "get_query_suggestions",
                "description": "Get smart query suggestions based on successful patterns",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "partial_query": {
                            "type": "string",
                            "description": "Partial or incomplete query for suggestions"
                        },
                        "max_suggestions": {
                            "type": "integer",
                            "description": "Maximum number of suggestions to return",
                            "default": 3
                        }
                    },
                    "required": ["partial_query"]
                }
            }),
            json!({
                "name": "get_database_stats",
                "description": "Get overall database statistics and performance metrics",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "include_performance": {
                            "type": "boolean",
                            "description": "Include query performance statistics",
                            "default": false
                        }
                    }
                }
            })
        ];
        
        Ok(json!({ "tools": tools }))
    }
    
    /// Handle tool call request with enhanced processing
    pub async fn handle_tool_call(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing tool call parameters"))?;
        
        let tool_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("Missing tool name"))?;
            
        let arguments = params.get("arguments")
            .ok_or_else(|| anyhow!("Missing tool arguments"))?;
        
        log::debug!("Executing tool: {} with args: {}", tool_name, arguments);
        
        let start_time = Instant::now();
        let result = match tool_name {
            "query_conversations" => self.tool_query_conversations(arguments).await,
            "get_speaker_analytics" => self.tool_get_speaker_analytics(arguments).await,
            "search_semantic" => self.tool_search_semantic(arguments).await,
            "get_conversation_context" => self.tool_get_conversation_context(arguments).await,
            "list_speakers" => self.tool_list_speakers(arguments).await,
            "learn_from_feedback" => self.tool_learn_from_feedback(arguments).await,
            "get_query_suggestions" => self.tool_get_query_suggestions(arguments).await,
            "get_database_stats" => self.tool_get_database_stats(arguments).await,
            _ => Err(anyhow!("Unknown tool: {}", tool_name)),
        };
        
        let execution_time = start_time.elapsed();
        log::debug!("Tool {} executed in {:?}", tool_name, execution_time);
        
        match result {
            Ok(content) => Ok(json!({
                "content": [
                    {
                        "type": "text",
                        "text": content
                    }
                ],
                "isError": false
            })),
            Err(e) => {
                log::error!("Tool {} failed: {}", tool_name, e);
                Ok(json!({
                    "content": [
                        {
                            "type": "text", 
                            "text": format!("Error: {}", e)
                        }
                    ],
                    "isError": true
                }))
            }
        }
    }
    
    /// Enhanced natural language query tool
    async fn tool_query_conversations(&self, args: &Value) -> Result<String> {
        let query = args.get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| anyhow!("Missing query parameter"))?;
            
        let session_id = args.get("session_id")
            .and_then(|s| s.as_str())
            .unwrap_or("default");
        
        // Validate query with enhanced security
        let sanitized_query = self.security.validate_natural_query(query)?;
        
        // Process query with LLM-powered understanding
        let llm_result = self.query_processor.process_query(&sanitized_query, session_id).await?;
        
        // Estimate complexity for security validation
        let complexity = self.security.estimate_query_cost(&llm_result.sql_query);
        
        // Enhanced security validation
        self.security.validate_query(&llm_result.sql_query, complexity).await?;
        
        // Execute query with proper error handling
        let results = self.execute_structured_query(&llm_result).await?;
        
        // Update conversation context
        let result_ids = self.extract_result_ids(&results);
        self.context_manager.update_context(session_id, query, &result_ids).await;
        
        // Format response for LLM consumption
        let formatted_result = self.format_query_results(&results, &llm_result, query).await?;
        
        Ok(formatted_result)
    }
    
    /// Get detailed speaker analytics with interaction data
    async fn tool_get_speaker_analytics(&self, args: &Value) -> Result<String> {
        let speaker = args.get("speaker")
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("Missing speaker parameter"))?;
            
        let include_interactions = args.get("include_interactions")
            .and_then(|i| i.as_bool())
            .unwrap_or(false);
        
        // Get basic speaker statistics
        let stats_query = r#"
            SELECT 
                speaker,
                conversation_count,
                total_segments,
                total_duration,
                avg_segment_duration,
                avg_confidence,
                first_appearance,
                last_appearance,
                active_days
            FROM speaker_analytics 
            WHERE speaker = ?
        "#;
        
        let stats = sqlx::query(stats_query)
            .bind(speaker)
            .fetch_optional(&self.database.pool)
            .await?;
        
        if stats.is_none() {
            return Ok(format!("No data found for speaker: {}", speaker));
        }
        
        let stats = stats.unwrap();
        let mut result = json!({
            "speaker": speaker,
            "conversation_count": stats.get::<i64, _>("conversation_count"),
            "total_segments": stats.get::<i64, _>("total_segments"),
            "total_duration_seconds": stats.get::<f64, _>("total_duration"),
            "avg_segment_duration": stats.get::<f64, _>("avg_segment_duration"),
            "avg_confidence": stats.get::<f64, _>("avg_confidence"),
            "first_appearance": stats.get::<chrono::DateTime<chrono::Utc>, _>("first_appearance"),
            "last_appearance": stats.get::<chrono::DateTime<chrono::Utc>, _>("last_appearance"),
            "active_days": stats.get::<i64, _>("active_days")
        });
        
        // Add interaction data if requested
        if include_interactions {
            let interactions_query = r#"
                SELECT speaker_a, speaker_b, interaction_count, total_duration, 
                       first_interaction, last_interaction
                FROM speaker_interaction_matrix 
                WHERE speaker_a = ? OR speaker_b = ?
                ORDER BY interaction_count DESC
            "#;
            
            let interactions = sqlx::query(interactions_query)
                .bind(speaker)
                .bind(speaker)
                .fetch_all(&self.database.pool)
                .await?;
            
            let interaction_data: Vec<Value> = interactions
                .into_iter()
                .map(|row| {
                    let other_speaker = if row.get::<String, _>("speaker_a") == speaker {
                        row.get::<String, _>("speaker_b")
                    } else {
                        row.get::<String, _>("speaker_a")
                    };
                    
                    json!({
                        "other_speaker": other_speaker,
                        "interaction_count": row.get::<i64, _>("interaction_count"),
                        "total_duration": row.get::<f64, _>("total_duration"),
                        "first_interaction": row.get::<chrono::DateTime<chrono::Utc>, _>("first_interaction"),
                        "last_interaction": row.get::<chrono::DateTime<chrono::Utc>, _>("last_interaction")
                    })
                })
                .collect();
            
            result.as_object_mut().unwrap().insert("interactions".to_string(), Value::Array(interaction_data));
        }
        
        Ok(serde_json::to_string_pretty(&result)?)
    }
    
    /// Enhanced semantic search with full-text search optimization
    async fn tool_search_semantic(&self, args: &Value) -> Result<String> {
        let term = args.get("term")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing search term"))?;
            
        let limit = args.get("limit")
            .and_then(|l| l.as_i64())
            .unwrap_or(20) as i32;
            
        let min_confidence = args.get("min_confidence")
            .and_then(|c| c.as_f64())
            .unwrap_or(0.0);
        
        // Use optimized FTS search
        let search_query = r#"
            SELECT 
                s.text,
                s.speaker,
                s.timestamp,
                s.confidence,
                c.title as conversation_title,
                c.id as conversation_id,
                rank
            FROM segments_fts 
            JOIN segments s ON segments_fts.rowid = s.id
            JOIN conversations c ON s.conversation_id = c.id
            WHERE segments_fts MATCH ? 
              AND s.confidence >= ?
            ORDER BY rank, s.timestamp DESC
            LIMIT ?
        "#;
        
        let results = sqlx::query(search_query)
            .bind(format!("\"{}\"", term)) // Exact phrase search
            .bind(min_confidence)
            .bind(limit)
            .fetch_all(&self.database.pool)
            .await?;
        
        if results.is_empty() {
            // Try fuzzy search if exact search fails
            let fuzzy_results = sqlx::query(search_query)
                .bind(term) // Fuzzy search
                .bind(min_confidence)
                .bind(limit)
                .fetch_all(&self.database.pool)
                .await?;
            
            if fuzzy_results.is_empty() {
                return Ok(format!("No results found for search term: {}", term));
            }
            
            let search_results: Vec<Value> = fuzzy_results
                .into_iter()
                .map(|row| {
                    json!({
                        "text": row.get::<String, _>("text"),
                        "speaker": row.get::<Option<String>, _>("speaker"),
                        "timestamp": row.get::<chrono::DateTime<chrono::Utc>, _>("timestamp"),
                        "confidence": row.get::<Option<f64>, _>("confidence"),
                        "conversation_title": row.get::<Option<String>, _>("conversation_title"),
                        "conversation_id": row.get::<String, _>("conversation_id"),
                        "match_type": "fuzzy"
                    })
                })
                .collect();
            
            return Ok(serde_json::to_string_pretty(&json!({
                "search_term": term,
                "results": search_results,
                "total_found": search_results.len(),
                "search_type": "fuzzy_fallback"
            }))?);
        }
        
        let search_results: Vec<Value> = results
            .into_iter()
            .map(|row| {
                json!({
                    "text": row.get::<String, _>("text"),
                    "speaker": row.get::<Option<String>, _>("speaker"),
                    "timestamp": row.get::<chrono::DateTime<chrono::Utc>, _>("timestamp"),
                    "confidence": row.get::<Option<f64>, _>("confidence"),
                    "conversation_title": row.get::<Option<String>, _>("conversation_title"),
                    "conversation_id": row.get::<String, _>("conversation_id"),
                    "match_type": "exact"
                })
            })
            .collect();
        
        Ok(serde_json::to_string_pretty(&json!({
            "search_term": term,
            "results": search_results,
            "total_found": search_results.len(),
            "search_type": "exact"
        }))?)
    }
    
    /// Get detailed conversation context
    async fn tool_get_conversation_context(&self, args: &Value) -> Result<String> {
        let conversation_id = args.get("conversation_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Missing conversation_id parameter"))?;
            
        let include_segments = args.get("include_segments")
            .and_then(|i| i.as_bool())
            .unwrap_or(true);
        
        // Get conversation details
        let conversation = self.database.get_conversation(conversation_id).await?;
        
        if conversation.is_none() {
            return Ok(format!("Conversation not found: {}", conversation_id));
        }
        
        let mut result = conversation.unwrap();
        
        if include_segments {
            let segments = self.database.get_conversation_segments(conversation_id).await?;
            result.as_object_mut().unwrap().insert("segments".to_string(), Value::Array(segments));
        }
        
        // Add summary statistics
        let stats_query = r#"
            SELECT 
                COUNT(*) as segment_count,
                COUNT(DISTINCT speaker) as unique_speakers,
                SUM(end_time - start_time) as total_duration,
                AVG(confidence) as avg_confidence,
                MIN(timestamp) as first_segment,
                MAX(timestamp) as last_segment
            FROM segments 
            WHERE conversation_id = ?
        "#;
        
        let stats = sqlx::query(stats_query)
            .bind(conversation_id)
            .fetch_one(&self.database.pool)
            .await?;
        
        result.as_object_mut().unwrap().insert("statistics".to_string(), json!({
            "segment_count": stats.get::<i64, _>("segment_count"),
            "unique_speakers": stats.get::<i64, _>("unique_speakers"),
            "total_duration": stats.get::<Option<f64>, _>("total_duration"),
            "avg_confidence": stats.get::<Option<f64>, _>("avg_confidence"),
            "first_segment": stats.get::<Option<chrono::DateTime<chrono::Utc>>, _>("first_segment"),
            "last_segment": stats.get::<Option<chrono::DateTime<chrono::Utc>>, _>("last_segment")
        }));
        
        Ok(serde_json::to_string_pretty(&result)?)
    }
    
    /// List speakers with enhanced statistics
    async fn tool_list_speakers(&self, args: &Value) -> Result<String> {
        let include_stats = args.get("include_stats")
            .and_then(|i| i.as_bool())
            .unwrap_or(true);
            
        let sort_by = args.get("sort_by")
            .and_then(|s| s.as_str())
            .unwrap_or("activity");
        
        let order_clause = match sort_by {
            "alphabetical" => "ORDER BY speaker ASC",
            "recent" => "ORDER BY last_appearance DESC",
            _ => "ORDER BY total_duration DESC", // activity (default)
        };
        
        let query = if include_stats {
            format!(r#"
                SELECT * FROM speaker_analytics 
                WHERE speaker IS NOT NULL AND speaker != ''
                {}
                LIMIT 50
            "#, order_clause)
        } else {
            format!(r#"
                SELECT DISTINCT speaker 
                FROM segments 
                WHERE speaker IS NOT NULL AND speaker != ''
                {}
                LIMIT 50
            "#, order_clause.replace("total_duration", "speaker"))
        };
        
        let speakers = sqlx::query(&query)
            .fetch_all(&self.database.pool)
            .await?;
        
        let speaker_data: Vec<Value> = if include_stats {
            speakers
                .into_iter()
                .map(|row| {
                    json!({
                        "speaker": row.get::<String, _>("speaker"),
                        "conversation_count": row.get::<i64, _>("conversation_count"),
                        "total_segments": row.get::<i64, _>("total_segments"),
                        "total_duration": row.get::<f64, _>("total_duration"),
                        "avg_confidence": row.get::<f64, _>("avg_confidence"),
                        "active_days": row.get::<i64, _>("active_days"),
                        "first_appearance": row.get::<chrono::DateTime<chrono::Utc>, _>("first_appearance"),
                        "last_appearance": row.get::<chrono::DateTime<chrono::Utc>, _>("last_appearance")
                    })
                })
                .collect()
        } else {
            speakers
                .into_iter()
                .map(|row| json!(row.get::<String, _>("speaker")))
                .collect()
        };
        
        Ok(serde_json::to_string_pretty(&json!({
            "speakers": speaker_data,
            "total_count": speaker_data.len(),
            "sorted_by": sort_by,
            "includes_stats": include_stats
        }))?)
    }
    
    /// Learn from user feedback for query optimization
    async fn tool_learn_from_feedback(&self, args: &Value) -> Result<String> {
        let query = args.get("query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| anyhow!("Missing query parameter"))?;
            
        let feedback_str = args.get("feedback")
            .and_then(|f| f.as_str())
            .ok_or_else(|| anyhow!("Missing feedback parameter"))?;
        
        let sql_query = args.get("sql_query")
            .and_then(|s| s.as_str())
            .unwrap_or("");
            
        let results = args.get("results")
            .unwrap_or(&json!({}));
        
        let feedback = match feedback_str {
            "Good" => UserFeedback::Good,
            "BadResults" => UserFeedback::BadResults,
            "TooSlow" => UserFeedback::TooSlow,
            "WrongIntent" => UserFeedback::WrongIntent,
            "Irrelevant" => UserFeedback::Irrelevant,
            _ => return Err(anyhow!("Invalid feedback type: {}", feedback_str)),
        };
        
        self.query_optimizer.learn_from_feedback(query, sql_query, results, feedback).await?;
        
        Ok(format!("Feedback recorded for query: {}", query))
    }
    
    /// Get query suggestions based on successful patterns
    async fn tool_get_query_suggestions(&self, args: &Value) -> Result<String> {
        let partial_query = args.get("partial_query")
            .and_then(|q| q.as_str())
            .ok_or_else(|| anyhow!("Missing partial_query parameter"))?;
            
        let max_suggestions = args.get("max_suggestions")
            .and_then(|m| m.as_i64())
            .unwrap_or(3) as usize;
        
        let suggestions = self.query_optimizer.get_query_suggestions(partial_query).await;
        
        Ok(serde_json::to_string_pretty(&json!({
            "partial_query": partial_query,
            "suggestions": suggestions.into_iter().take(max_suggestions).collect::<Vec<_>>(),
            "suggestion_count": suggestions.len().min(max_suggestions)
        }))?)
    }
    
    /// Get database statistics and performance metrics
    async fn tool_get_database_stats(&self, args: &Value) -> Result<String> {
        let include_performance = args.get("include_performance")
            .and_then(|p| p.as_bool())
            .unwrap_or(false);
        
        // Get basic database statistics
        let stats_query = r#"
            SELECT 
                (SELECT COUNT(*) FROM conversations) as total_conversations,
                (SELECT COUNT(*) FROM segments) as total_segments,
                (SELECT COUNT(DISTINCT speaker) FROM segments WHERE speaker IS NOT NULL) as unique_speakers,
                (SELECT SUM(end_time - start_time) FROM segments WHERE end_time IS NOT NULL AND start_time IS NOT NULL) as total_duration,
                (SELECT MIN(start_time) FROM conversations) as earliest_conversation,
                (SELECT MAX(start_time) FROM conversations) as latest_conversation
        "#;
        
        let stats = sqlx::query(stats_query)
            .fetch_one(&self.database.pool)
            .await?;
        
        let mut result = json!({
            "total_conversations": stats.get::<i64, _>("total_conversations"),
            "total_segments": stats.get::<i64, _>("total_segments"),
            "unique_speakers": stats.get::<i64, _>("unique_speakers"),
            "total_duration_seconds": stats.get::<Option<f64>, _>("total_duration"),
            "earliest_conversation": stats.get::<Option<chrono::DateTime<chrono::Utc>>, _>("earliest_conversation"),
            "latest_conversation": stats.get::<Option<chrono::DateTime<chrono::Utc>>, _>("latest_conversation")
        });
        
        if include_performance {
            let perf_query = r#"
                SELECT 
                    query_type,
                    COUNT(*) as query_count,
                    AVG(execution_time_ms) as avg_execution_time,
                    MAX(execution_time_ms) as max_execution_time,
                    SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful_queries
                FROM query_performance 
                WHERE timestamp > datetime('now', '-24 hours')
                GROUP BY query_type
                ORDER BY query_count DESC
            "#;
            
            let perf_stats = sqlx::query(perf_query)
                .fetch_all(&self.database.pool)
                .await
                .unwrap_or_default();
            
            let performance_data: Vec<Value> = perf_stats
                .into_iter()
                .map(|row| {
                    json!({
                        "query_type": row.get::<String, _>("query_type"),
                        "query_count": row.get::<i64, _>("query_count"),
                        "avg_execution_time_ms": row.get::<f64, _>("avg_execution_time"),
                        "max_execution_time_ms": row.get::<i64, _>("max_execution_time"),
                        "successful_queries": row.get::<i64, _>("successful_queries"),
                        "success_rate": row.get::<i64, _>("successful_queries") as f64 / row.get::<i64, _>("query_count") as f64
                    })
                })
                .collect();
            
            result.as_object_mut().unwrap().insert("performance_24h".to_string(), Value::Array(performance_data));
        }
        
        Ok(serde_json::to_string_pretty(&result)?)
    }
    
    // Helper methods
    
    async fn execute_structured_query(&self, llm_result: &crate::natural_query::LLMQueryResult) -> Result<Value> {
        // This would execute the SQL query from the LLM result
        // For now, return a placeholder
        Ok(json!({
            "intent": llm_result.intent,
            "confidence": llm_result.confidence,
            "results": []
        }))
    }
    
    fn extract_result_ids(&self, results: &Value) -> Vec<String> {
        // Extract IDs from results for context management
        vec![]
    }
    
    async fn format_query_results(&self, results: &Value, llm_result: &crate::natural_query::LLMQueryResult, original_query: &str) -> Result<String> {
        // Format results for LLM consumption
        Ok(serde_json::to_string_pretty(&json!({
            "original_query": original_query,
            "intent": llm_result.intent,
            "confidence": llm_result.confidence,
            "results": results,
            "summary": format!("Query executed successfully with {} confidence", llm_result.confidence)
        }))?)
    }
}