//! Security layer for database query validation and access control
//! 
//! Provides query validation, input sanitization, and access control for LLM integration

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashMap;
use std::collections::HashSet;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use sqlparser::ast::{Statement, Query, SetExpr, SelectItem, TableFactor, Expr};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

/// Query complexity levels for rate limiting
#[derive(Debug, Clone, PartialEq)]
pub enum QueryComplexity {
    Low,
    Medium,
    High,
}

/// Rate limiter for preventing abuse
pub struct RateLimiter {
    query_history: Arc<RwLock<HashMap<String, Vec<Instant>>>>,
    max_queries_per_minute: usize,
    max_complexity_per_minute: usize,
}

/// Enhanced security manager for validating and controlling database access
pub struct QuerySecurityManager {
    /// Tables that can be queried
    pub allowed_tables: HashSet<String>,
    /// Maximum number of results per query
    pub max_result_limit: usize,
    /// Query execution timeout
    pub query_timeout: Duration,
    /// Whether to allow complex joins
    pub allow_joins: bool,
    /// Regex patterns for dangerous operations
    dangerous_patterns: Vec<Regex>,
    /// Rate limiter for query frequency
    rate_limiter: RateLimiter,
    /// Patterns that could indicate timing attacks
    timing_attack_patterns: Vec<Regex>,
    /// Patterns that suggest string concatenation (injection risk)
    concatenation_patterns: Vec<Regex>,
}

/// Errors that can occur during security validation
#[derive(thiserror::Error, Debug)]
pub enum SecurityError {
    #[error("Query contains dangerous operations: {operation}")]
    DangerousOperation { operation: String },
    
    #[error("Table '{table}' is not allowed")]
    UnauthorizedTable { table: String },
    
    #[error("Result limit {limit} exceeds maximum {max_limit}")]
    ExcessiveResultLimit { limit: usize, max_limit: usize },
    
    #[error("Query is too long: {length} characters (max: {max_length})")]
    QueryTooLong { length: usize, max_length: usize },
    
    #[error("Query contains invalid characters")]
    InvalidCharacters,
    
    #[error("Query parsing failed: {reason}")]
    ParseError { reason: String },
    
    #[error("Non-SELECT operations are not allowed")]
    NonSelectOperation,
    
    #[error("Complex joins are not allowed")]
    ComplexJoinsNotAllowed,
    
    #[error("Sensitive content detected")]
    SensitiveContent,
    
    #[error("Timing attack pattern detected")]
    TimingAttack,
    
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    
    #[error("String concatenation detected - requires parameterization")]
    RequiresParameterization,
    
    #[error("Invalid SQL structure")]
    InvalidSQL,
}

impl Default for QuerySecurityManager {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiter {
    pub fn new() -> Self {
        Self {
            query_history: Arc::new(RwLock::new(HashMap::new())),
            max_queries_per_minute: 60,
            max_complexity_per_minute: 100,
        }
    }
    
    pub async fn check_rate_limit(&self, client_id: &str) -> bool {
        let now = Instant::now();
        let mut history = self.query_history.write().await;
        
        let client_queries = history.entry(client_id.to_string()).or_insert_with(Vec::new);
        
        // Remove queries older than 1 minute
        client_queries.retain(|&timestamp| now.duration_since(timestamp) < Duration::from_secs(60));
        
        if client_queries.len() >= self.max_queries_per_minute {
            return false;
        }
        
        client_queries.push(now);
        true
    }
    
    pub async fn check_complexity(&self, complexity: QueryComplexity) -> bool {
        // Simplified complexity check - in production this would track complexity points
        match complexity {
            QueryComplexity::High => self.check_rate_limit("complexity_high").await,
            QueryComplexity::Medium => true, // Allow medium complexity more freely
            QueryComplexity::Low => true,
        }
    }
}

impl QuerySecurityManager {
    /// Create a new security manager with default settings
    pub fn new() -> Self {
        let mut allowed_tables = HashSet::new();
        allowed_tables.insert("conversations".to_string());
        allowed_tables.insert("segments".to_string());
        allowed_tables.insert("speakers".to_string());
        allowed_tables.insert("speaker_relationships".to_string());
        allowed_tables.insert("segments_fts".to_string());
        allowed_tables.insert("query_history".to_string());
        
        let dangerous_patterns = vec![
            Regex::new(r"(?i)\b(drop|delete|update|insert|create|alter|truncate)\b").unwrap(),
            Regex::new(r"(?i)\b(exec|execute|sp_|xp_)\b").unwrap(),
            Regex::new(r"(?i)\b(union|intersect|except)\b").unwrap(),
            Regex::new(r"(?i)\b(declare|cursor|while|if)\b").unwrap(),
            Regex::new(r"(?i)--").unwrap(), // SQL comments
            Regex::new(r"(?i)/\*").unwrap(), // Block comments
            Regex::new(r"(?i);").unwrap(), // Multiple statements
        ];
        
        let timing_attack_patterns = vec![
            Regex::new(r"(?i)\bsleep\s*\(").unwrap(),
            Regex::new(r"(?i)\bwaitfor\s+delay").unwrap(),
            Regex::new(r"(?i)\bbenchmark\s*\(").unwrap(),
            Regex::new(r"(?i)\bheavy_computation\s*\(").unwrap(),
        ];
        
        let concatenation_patterns = vec![
            Regex::new(r"(?i)\|\|").unwrap(), // String concatenation
            Regex::new(r"(?i)\+.*['\"]").unwrap(), // Potential string concatenation
            Regex::new(r"(?i)concat\s*\(").unwrap(), // CONCAT function
        ];
        
        Self {
            allowed_tables,
            max_result_limit: 1000,
            query_timeout: Duration::from_secs(30),
            allow_joins: true,
            dangerous_patterns,
            rate_limiter: RateLimiter::new(),
            timing_attack_patterns,
            concatenation_patterns,
        }
    }
    
    /// Create a read-only security manager for MCP server
    pub fn read_only() -> Self {
        let mut manager = Self::new();
        manager.max_result_limit = 100;
        manager.query_timeout = Duration::from_secs(5);
        manager.allow_joins = false;
        manager
    }
    
    /// Validate a natural language query input
    pub fn validate_natural_query(&self, query: &str) -> Result<String, SecurityError> {
        const MAX_QUERY_LENGTH: usize = 1000;
        
        // Length check
        if query.len() > MAX_QUERY_LENGTH {
            return Err(SecurityError::QueryTooLong {
                length: query.len(),
                max_length: MAX_QUERY_LENGTH,
            });
        }
        
        // Character validation
        let allowed_chars = query.chars().all(|c| {
            c.is_alphanumeric() 
                || " .,?!-_()[]{}\"'".contains(c)
                || c.is_whitespace()
        });
        
        if !allowed_chars {
            return Err(SecurityError::InvalidCharacters);
        }
        
        // Content filtering
        if self.contains_sensitive_patterns(query) {
            return Err(SecurityError::SensitiveContent);
        }
        
        // Return sanitized query
        Ok(self.sanitize_natural_query(query))
    }
    
    /// Enhanced validation for generated SQL queries
    pub async fn validate_query(&self, query: &str, complexity: QueryComplexity) -> Result<(), SecurityError> {
        // 1. Parse SQL to AST
        let ast = self.parse_and_validate_sql(query)?;
        
        // 2. Validate SELECT-only operations
        self.ensure_read_only(&ast)?;
        
        // 3. Check table whitelist
        self.validate_table_access_ast(&ast)?;
        
        // 4. Validate result limits
        self.check_result_limits(&ast)?;
        
        // 5. Check for sensitive data access
        self.validate_column_access(&ast)?;
        
        // 6. Prevent timing-based attacks
        if self.contains_timing_attack_patterns(query) {
            return Err(SecurityError::TimingAttack);
        }
        
        // 7. Rate limiting per query complexity
        if !self.rate_limiter.check_complexity(complexity).await {
            return Err(SecurityError::RateLimitExceeded);
        }
        
        // 8. Enforce parameterized queries
        if self.contains_string_concatenation(query) {
            return Err(SecurityError::RequiresParameterization);
        }
        
        Ok(())
    }
    
    /// Parse SQL and perform basic validation
    fn parse_and_validate_sql(&self, sql: &str) -> Result<Vec<Statement>, SecurityError> {
        let dialect = SQLiteDialect {};
        let statements = Parser::parse_sql(&dialect, sql)
            .map_err(|_| SecurityError::InvalidSQL)?;
        
        if statements.len() != 1 {
            return Err(SecurityError::DangerousOperation {
                operation: "Multiple statements".to_string(),
            });
        }
        
        Ok(statements)
    }
    
    /// Ensure query only contains read operations
    fn ensure_read_only(&self, statements: &[Statement]) -> Result<(), SecurityError> {
        for statement in statements {
            match statement {
                Statement::Query(_) => {}, // SELECT is OK
                _ => return Err(SecurityError::NonSelectOperation),
            }
        }
        Ok(())
    }
    
    /// Validate table access using AST
    fn validate_table_access_ast(&self, statements: &[Statement]) -> Result<(), SecurityError> {
        for statement in statements {
            if let Statement::Query(query) = statement {
                self.validate_select_query(query)?;
            }
        }
        Ok(())
    }
    
    /// Check result limits in AST
    fn check_result_limits(&self, statements: &[Statement]) -> Result<(), SecurityError> {
        for statement in statements {
            if let Statement::Query(query) = statement {
                if let Some(limit) = &query.limit {
                    if let Expr::Value(sqlparser::ast::Value::Number(limit_str, _)) = limit {
                        if let Ok(limit_num) = limit_str.parse::<usize>() {
                            if limit_num > self.max_result_limit {
                                return Err(SecurityError::ExcessiveResultLimit {
                                    limit: limit_num,
                                    max_limit: self.max_result_limit,
                                });
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Validate column access permissions
    fn validate_column_access(&self, _statements: &[Statement]) -> Result<(), SecurityError> {
        // In a production system, this would check for sensitive columns
        // For now, we allow all columns on allowed tables
        Ok(())
    }
    
    /// Check for timing attack patterns
    fn contains_timing_attack_patterns(&self, query: &str) -> bool {
        self.timing_attack_patterns.iter().any(|pattern| pattern.is_match(query))
    }
    
    /// Check for string concatenation patterns
    fn contains_string_concatenation(&self, query: &str) -> bool {
        self.concatenation_patterns.iter().any(|pattern| pattern.is_match(query))
    }
    
    /// Estimate query complexity for rate limiting
    pub fn estimate_query_cost(&self, query: &str) -> QueryComplexity {
        let complexity_score = query.matches("JOIN").count() * 2 +
                              query.matches("ORDER BY").count() * 1 +
                              query.matches("GROUP BY").count() * 2 +
                              query.matches("LIKE").count() * 1 +
                              query.matches("MATCH").count() * 3; // FTS queries are expensive
        
        match complexity_score {
            0..=2 => QueryComplexity::Low,
            3..=5 => QueryComplexity::Medium,
            _ => QueryComplexity::High,
        }
    }
    
    /// Legacy SQL validation method for backward compatibility
    pub fn validate_sql_query(&self, sql: &str) -> Result<(), SecurityError> {
        let complexity = self.estimate_query_cost(sql);
        // Note: This is a blocking version - in practice use validate_query
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(self.validate_query(sql, complexity))
        })
    }
    
    /// Validate a SELECT query AST
    fn validate_select_query(&self, query: &Query) -> Result<(), SecurityError> {
        match &*query.body {
            SetExpr::Select(select) => {
                // Validate table access
                for table in &select.from {
                    self.validate_table_access(&table.relation)?;
                    
                    // Check joins if not allowed
                    if !self.allow_joins && !table.joins.is_empty() {
                        return Err(SecurityError::ComplexJoinsNotAllowed);
                    }
                }
                
                // Validate LIMIT clause
                if let Some(limit) = &query.limit {
                    if let Expr::Value(sqlparser::ast::Value::Number(limit_str, _)) = limit {
                        if let Ok(limit_num) = limit_str.parse::<usize>() {
                            if limit_num > self.max_result_limit {
                                return Err(SecurityError::ExcessiveResultLimit {
                                    limit: limit_num,
                                    max_limit: self.max_result_limit,
                                });
                            }
                        }
                    }
                }
                
                // Add default limit if none specified
                if query.limit.is_none() {
                    // This would need to be handled at the query building level
                }
            }
            _ => {
                return Err(SecurityError::DangerousOperation {
                    operation: "Non-SELECT query body".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Validate table access permissions
    fn validate_table_access(&self, table: &TableFactor) -> Result<(), SecurityError> {
        match table {
            TableFactor::Table { name, .. } => {
                let table_name = name.0.first()
                    .map(|ident| ident.value.clone())
                    .unwrap_or_default();
                
                if !self.allowed_tables.contains(&table_name) {
                    return Err(SecurityError::UnauthorizedTable { 
                        table: table_name 
                    });
                }
            }
            _ => {
                return Err(SecurityError::DangerousOperation {
                    operation: "Complex table expressions not allowed".to_string(),
                });
            }
        }
        
        Ok(())
    }
    
    /// Sanitize natural language input
    fn sanitize_natural_query(&self, input: &str) -> String {
        // Remove potentially dangerous characters and normalize whitespace
        input
            .chars()
            .filter(|&c| {
                c.is_alphanumeric() 
                    || " .,?!-_()[]{}\"'".contains(c)
                    || c.is_whitespace()
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .trim()
            .to_string()
    }
    
    /// Check for sensitive content patterns
    fn contains_sensitive_patterns(&self, query: &str) -> bool {
        let sensitive_patterns = [
            "password", "secret", "key", "token", "auth",
            "credit card", "ssn", "social security",
            "personal", "private", "confidential",
        ];
        
        let query_lower = query.to_lowercase();
        sensitive_patterns.iter().any(|pattern| query_lower.contains(pattern))
    }
    
    /// Add a safe default LIMIT to queries that don't have one
    pub fn ensure_query_limit(&self, sql: &str) -> String {
        if !sql.to_uppercase().contains("LIMIT") {
            format!("{} LIMIT {}", sql, self.max_result_limit)
        } else {
            sql.to_string()
        }
    }
    
    /// Get maximum allowed result limit
    pub fn max_result_limit(&self) -> usize {
        self.max_result_limit
    }
    
    /// Get query timeout duration
    pub fn query_timeout(&self) -> Duration {
        self.query_timeout
    }
}

/// Sanitize and validate user input for database operations
pub fn sanitize_input(input: &str) -> Result<String> {
    if input.is_empty() {
        return Err(anyhow!("Input cannot be empty"));
    }
    
    if input.len() > 10000 {
        return Err(anyhow!("Input too long"));
    }
    
    // Remove null bytes and control characters
    let sanitized = input
        .chars()
        .filter(|&c| c != '\0' && !c.is_control() || c.is_whitespace())
        .collect::<String>();
    
    Ok(sanitized)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_natural_query_validation() {
        let security = QuerySecurityManager::new();
        
        // Valid queries
        assert!(security.validate_natural_query("Find conversations with John").is_ok());
        assert!(security.validate_natural_query("Show me talks from last week").is_ok());
        
        // Invalid queries
        assert!(security.validate_natural_query("DROP TABLE conversations").is_err());
        assert!(security.validate_natural_query("Show me all passwords").is_err());
        
        // Too long query
        let long_query = "a".repeat(1001);
        assert!(security.validate_natural_query(&long_query).is_err());
    }
    
    #[test]
    fn test_sql_validation() {
        let security = QuerySecurityManager::new();
        
        // Valid SQL
        assert!(security.validate_sql_query(
            "SELECT * FROM conversations WHERE speaker = 'john' LIMIT 10"
        ).is_ok());
        
        // Invalid SQL
        assert!(security.validate_sql_query("DROP TABLE conversations").is_err());
        assert!(security.validate_sql_query("UPDATE conversations SET title = 'hacked'").is_err());
        assert!(security.validate_sql_query("SELECT * FROM unauthorized_table").is_err());
    }
    
    #[test]
    fn test_input_sanitization() {
        assert_eq!(sanitize_input("normal text").unwrap(), "normal text");
        assert!(sanitize_input("").is_err());
        assert!(sanitize_input(&"x".repeat(10001)).is_err());
    }
}