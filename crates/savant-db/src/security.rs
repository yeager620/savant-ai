//! Security layer for database query validation and access control
//! 
//! Provides query validation, input sanitization, and access control for LLM integration

use anyhow::{anyhow, Result};
use regex::Regex;
use std::collections::HashSet;
use std::time::Duration;
use sqlparser::ast::{Statement, Query, SetExpr, SelectItem, TableFactor, Expr};
use sqlparser::dialect::SQLiteDialect;
use sqlparser::parser::Parser;

/// Security manager for validating and controlling database access
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
}

impl Default for QuerySecurityManager {
    fn default() -> Self {
        Self::new()
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
        
        Self {
            allowed_tables,
            max_result_limit: 1000,
            query_timeout: Duration::from_secs(30),
            allow_joins: true,
            dangerous_patterns,
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
    
    /// Validate a generated SQL query
    pub fn validate_sql_query(&self, sql: &str) -> Result<(), SecurityError> {
        // Parse SQL to ensure it's valid and safe
        let dialect = SQLiteDialect {};
        let statements = Parser::parse_sql(&dialect, sql)
            .map_err(|e| SecurityError::ParseError { 
                reason: e.to_string() 
            })?;
        
        if statements.len() != 1 {
            return Err(SecurityError::DangerousOperation {
                operation: "Multiple statements".to_string(),
            });
        }
        
        let statement = &statements[0];
        
        // Only allow SELECT statements
        match statement {
            Statement::Query(query) => self.validate_select_query(query)?,
            _ => return Err(SecurityError::NonSelectOperation),
        }
        
        // Check for dangerous patterns in raw SQL
        for pattern in &self.dangerous_patterns {
            if pattern.is_match(sql) {
                return Err(SecurityError::DangerousOperation {
                    operation: format!("Pattern match: {}", pattern.as_str()),
                });
            }
        }
        
        Ok(())
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