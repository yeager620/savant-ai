//! LLM database integration commands for Tauri
//! 
//! Provides natural language querying capabilities for the conversation database

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;

use savant_db::{TranscriptDatabase, MCPServer, NaturalLanguageQueryParser, QueryResult};

/// Shared MCP server state
pub type MCPServerState = Arc<Mutex<Option<MCPServer>>>;

/// Request for natural language database query
#[derive(Debug, Deserialize)]
pub struct NaturalQueryRequest {
    pub query: String,
}

/// Response from natural language database query
#[derive(Debug, Serialize)]
pub struct NaturalQueryResponse {
    pub success: bool,
    pub results: serde_json::Value,
    pub summary: String,
    pub execution_time_ms: u64,
    pub intent_type: String,
    pub result_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// MCP server status
#[derive(Debug, Serialize)]
pub struct MCPServerStatus {
    pub running: bool,
    pub session_count: usize,
    pub last_activity: Option<chrono::DateTime<chrono::Utc>>,
}

/// Execute a natural language query against the database
#[tauri::command]
pub async fn natural_language_query(
    query: String,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<NaturalQueryResponse, String> {
    // Create query parser
    let parser = NaturalLanguageQueryParser::new(database.pool.clone());
    
    // Execute the query
    match parser.execute_natural_query(&query).await {
        Ok(result) => {
            let summary = format_query_summary(&result);
            
            Ok(NaturalQueryResponse {
                success: true,
                results: result.results.clone(),
                summary,
                execution_time_ms: result.execution_time_ms,
                intent_type: result.intent.intent_type.to_string(),
                result_count: result.result_count,
                error: None,
            })
        }
        Err(e) => Ok(NaturalQueryResponse {
            success: false,
            results: serde_json::Value::Null,
            summary: format!("Query failed: {}", e),
            execution_time_ms: 0,
            intent_type: "error".to_string(),
            result_count: 0,
            error: Some(e.to_string()),
        })
    }
}

/// Start the MCP server for external LLM integration
#[tauri::command]
pub async fn start_mcp_server(
    database: State<'_, Arc<TranscriptDatabase>>,
    mcp_server_state: State<'_, MCPServerState>,
) -> Result<String, String> {
    let mut server_guard = mcp_server_state.lock().await;
    
    if server_guard.is_some() {
        return Ok("MCP server already running".to_string());
    }
    
    // Create MCP server
    let mcp_server = MCPServer::new(database.inner().clone())
        .map_err(|e| format!("Failed to create MCP server: {}", e))?;
    
    // Store server instance
    *server_guard = Some(mcp_server);
    
    // Note: In a real implementation, you'd start the server in a background task
    // For now, we just initialize it for potential use
    
    Ok("MCP server initialized successfully".to_string())
}

/// Get MCP server status
#[tauri::command]
pub async fn get_mcp_server_status(
    mcp_server_state: State<'_, MCPServerState>,
) -> Result<MCPServerStatus, String> {
    let server_guard = mcp_server_state.lock().await;
    
    Ok(MCPServerStatus {
        running: server_guard.is_some(),
        session_count: 0, // Would be tracked in a real implementation
        last_activity: None, // Would be tracked in a real implementation
    })
}

/// Test database connection and basic functionality
#[tauri::command]
pub async fn test_database_connection(
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    // Test basic database operations
    match database.get_speaker_stats().await {
        Ok(stats) => {
            let conversations = database.list_conversations(Some(5)).await
                .unwrap_or_default();
            
            Ok(serde_json::json!({
                "success": true,
                "speaker_count": stats.len(),
                "conversation_count": conversations.len(),
                "database_path": "Connected successfully",
                "capabilities": [
                    "natural_language_queries",
                    "speaker_analytics", 
                    "semantic_search",
                    "conversation_analysis"
                ]
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

/// Get database statistics for the frontend
#[tauri::command]
pub async fn get_database_stats(
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    let stats = database.get_speaker_stats().await
        .map_err(|e| e.to_string())?;
    
    let conversations = database.list_conversations(Some(1)).await
        .map_err(|e| e.to_string())?;
    
    let total_duration: f64 = stats.iter().map(|s| s.total_duration_seconds).sum();
    let total_conversations: i64 = stats.iter().map(|s| s.conversation_count).sum();
    let total_segments: i64 = stats.iter().map(|s| s.total_segments).sum();
    
    Ok(serde_json::json!({
        "total_speakers": stats.len(),
        "total_conversations": total_conversations,
        "total_segments": total_segments,
        "total_duration_hours": total_duration / 3600.0,
        "average_conversation_length_minutes": if total_conversations > 0 {
            (total_duration / 60.0) / total_conversations as f64
        } else {
            0.0
        },
        "top_speakers": stats.into_iter().take(5).map(|s| serde_json::json!({
            "name": s.speaker,
            "conversation_count": s.conversation_count,
            "total_duration_hours": s.total_duration_seconds / 3600.0
        })).collect::<Vec<_>>()
    }))
}

/// Search conversations with natural language
#[tauri::command]
pub async fn search_conversations(
    query: String,
    limit: Option<usize>,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    let search_limit = limit.unwrap_or(20);
    
    // Try semantic search first, fall back to text search
    let results = database.text_search(&query, search_limit).await
        .map_err(|e| e.to_string())?;
    
    let formatted_results: Vec<serde_json::Value> = results.into_iter().map(|result| {
        serde_json::json!({
            "text": result.text,
            "speaker": result.speaker,
            "timestamp": result.timestamp,
            "confidence": result.confidence,
            "conversation_id": result.conversation_id
        })
    }).collect();
    
    Ok(serde_json::json!({
        "query": query,
        "results": formatted_results,
        "result_count": formatted_results.len()
    }))
}

/// Get conversation analysis
#[tauri::command]
pub async fn analyze_conversation(
    conversation_id: String,
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<serde_json::Value, String> {
    match database.analyze_conversation(&conversation_id).await {
        Ok(analysis) => {
            Ok(serde_json::json!({
                "success": true,
                "analysis": {
                    "topics": analysis.topics,
                    "sentiment_score": analysis.sentiment_score,
                    "total_duration": analysis.total_duration,
                    "participants": analysis.participants,
                    "summary": analysis.summary
                }
            }))
        }
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))
    }
}

/// List available speakers with statistics
#[tauri::command]
pub async fn list_speakers_with_stats(
    database: State<'_, Arc<TranscriptDatabase>>,
) -> Result<Vec<serde_json::Value>, String> {
    let stats = database.get_speaker_stats().await
        .map_err(|e| e.to_string())?;
    
    Ok(stats.into_iter().map(|s| serde_json::json!({
        "name": s.speaker,
        "conversation_count": s.conversation_count,
        "total_duration_seconds": s.total_duration_seconds,
        "total_segments": s.total_segments,
        "avg_confidence": s.avg_confidence
    })).collect())
}

/// Format query result into human-readable summary
fn format_query_summary(result: &QueryResult) -> String {
    let intent_desc = match result.intent.intent_type {
        savant_db::IntentType::FindConversations => "Found Conversations",
        savant_db::IntentType::AnalyzeSpeaker => "Speaker Analysis", 
        savant_db::IntentType::SearchContent => "Content Search",
        savant_db::IntentType::GetStatistics => "Database Statistics",
        savant_db::IntentType::ListSpeakers => "Speaker List",
        savant_db::IntentType::GetTopics => "Topic Analysis",
        _ => "Query Results",
    };
    
    format!(
        "{}: Found {} results in {}ms for \"{}\"",
        intent_desc,
        result.result_count,
        result.execution_time_ms,
        result.intent.original_query
    )
}