use anyhow::Result;
use serde_json::{json, Value};
use savant_mcp::{MCPServer, MCPRequest, MCPResponse};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use tempfile::TempDir;
use chrono::Utc;
use std::sync::Arc;

async fn setup_test_database_schema(pool: &Pool<Sqlite>) -> Result<()> {
    // Create core conversation tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS conversations (
            id TEXT PRIMARY KEY,
            title TEXT,
            start_time DATETIME NOT NULL,
            end_time DATETIME,
            context TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS segments (
            id TEXT PRIMARY KEY,
            conversation_id TEXT NOT NULL,
            timestamp DATETIME NOT NULL,
            speaker TEXT NOT NULL,
            audio_source TEXT NOT NULL,
            text TEXT NOT NULL,
            start_time REAL NOT NULL,
            end_time REAL NOT NULL,
            confidence REAL,
            metadata TEXT,
            created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (conversation_id) REFERENCES conversations(id) ON DELETE CASCADE
        )
        "#
    ).execute(pool).await?;

    // Create high-frequency video capture tables
    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hf_video_frames (
            id TEXT PRIMARY KEY DEFAULT (hex(randomblob(16))),
            timestamp_ms INTEGER NOT NULL,
            session_id TEXT NOT NULL,
            frame_hash TEXT NOT NULL,
            change_score REAL DEFAULT 0.0,
            file_path TEXT,
            screen_resolution TEXT,
            active_app TEXT,
            processing_flags INTEGER DEFAULT 0,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
        )
        "#
    ).execute(pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hf_text_extractions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            frame_id TEXT NOT NULL,
            word_text TEXT NOT NULL,
            confidence REAL NOT NULL,
            bbox_x INTEGER NOT NULL,
            bbox_y INTEGER NOT NULL,
            bbox_width INTEGER NOT NULL,
            bbox_height INTEGER NOT NULL,
            font_size_estimate INTEGER,
            text_type TEXT,
            line_id INTEGER,
            paragraph_id INTEGER,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id)
        )
        "#
    ).execute(pool).await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS hf_detected_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            frame_id TEXT NOT NULL,
            task_type TEXT NOT NULL,
            confidence REAL NOT NULL,
            description TEXT,
            evidence_text TEXT,
            bounding_regions TEXT,
            assistance_suggestions TEXT,
            created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (frame_id) REFERENCES hf_video_frames(id)
        )
        "#
    ).execute(pool).await?;

    Ok(())
}

async fn setup_test_server() -> Result<(Arc<MCPServer>, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite://{}?mode=rwc", db_path.display()))
        .await?;

    // Setup test database schema manually
    setup_test_database_schema(&pool).await?;

    // Insert test data
    sqlx::query(
        r#"
        INSERT INTO conversations (id, start_time, title)
        VALUES ('conv-1', datetime('now', '-1 hour'), 'Test Conversation')
        "#
    ).execute(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO segments (id, conversation_id, timestamp, speaker, audio_source, text, start_time, end_time, confidence)
        VALUES 
            ('seg-1', 'conv-1', datetime('now', '-30 minutes'), 'user', 'microphone', 'How do I implement a binary search tree?', 0.0, 5.0, 0.95),
            ('seg-2', 'conv-1', datetime('now', '-29 minutes'), 'assistant', 'system', 'To implement a binary search tree, you need to create a Node class...', 0.0, 15.0, 0.98),
            ('seg-3', 'conv-1', datetime('now', '-15 minutes'), 'user', 'microphone', 'Can you show me the code in Python?', 0.0, 3.0, 0.93),
            ('seg-4', 'conv-1', datetime('now', '-14 minutes'), 'assistant', 'system', 'Here is a Python implementation of a binary search tree...', 0.0, 20.0, 0.97)
        "#
    ).execute(&pool).await?;

    // Add visual data with proper frame IDs
    let frame_1_id = "frame-1-id";
    let frame_2_id = "frame-2-id"; 
    let frame_3_id = "frame-3-id";
    
    sqlx::query(
        r#"
        INSERT INTO hf_video_frames (id, timestamp_ms, session_id, frame_hash, change_score, active_app)
        VALUES 
            (?, ?, 'session-1', 'frame-1', 0.9, 'Visual Studio Code'),
            (?, ?, 'session-1', 'frame-2', 0.85, 'Chrome'),
            (?, ?, 'session-1', 'frame-3', 0.7, 'Terminal')
        "#
    )
    .bind(frame_1_id)
    .bind(Utc::now().timestamp_millis() - 1800000)
    .bind(frame_2_id)
    .bind(Utc::now().timestamp_millis() - 900000)
    .bind(frame_3_id)
    .bind(Utc::now().timestamp_millis() - 300000)
    .execute(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO hf_text_extractions (frame_id, word_text, confidence, bbox_x, bbox_y, bbox_width, bbox_height, text_type)
        VALUES 
            (?, 'class', 0.98, 100, 200, 50, 20, 'CodeSnippet'),
            (?, 'BinarySearchTree', 0.97, 150, 200, 150, 20, 'CodeSnippet'),
            (?, 'LeetCode', 0.99, 500, 100, 100, 30, 'UIElement'),
            (?, 'python', 0.95, 200, 300, 60, 18, 'CommandLine')
        "#
    )
    .bind(frame_1_id)
    .bind(frame_1_id)
    .bind(frame_2_id)
    .bind(frame_3_id)
    .execute(&pool).await?;

    sqlx::query(
        r#"
        INSERT INTO hf_detected_tasks (frame_id, task_type, confidence, description, evidence_text)
        VALUES 
            (?, 'CodingProblem', 0.92, 'Implementing binary search tree', '{"language": "Python", "complexity": "Medium"}')
        "#
    )
    .bind(frame_1_id)
    .execute(&pool).await?;

    let database = Arc::new(savant_db::TranscriptDatabase::new(Some(db_path.clone())).await?);
    let server = Arc::new(MCPServer::new(database, None).await?);
    Ok((server, temp_dir))
}

#[tokio::test]
async fn test_list_tools() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolsList { result, .. } = response {
        let tools = result.as_array().expect("result should be an array");
        assert!(!tools.is_empty());

        // Check that key tools are present
        let tool_names: Vec<String> = tools.iter()
            .filter_map(|t| t.get("name")?.as_str())
            .map(|s| s.to_string())
            .collect();
        assert!(tool_names.contains(&"query_conversations".to_string()));
        assert!(tool_names.contains(&"search_semantic".to_string()));
        assert!(tool_names.contains(&"get_current_activity".to_string()));
        assert!(tool_names.contains(&"query_multimodal_context".to_string()));
    } else {
        panic!("Expected ToolsList response");
    }
}

#[tokio::test]
async fn test_query_conversations_natural_language() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    // Test various natural language queries
    let queries = vec![
        ("Show me conversations about binary search trees", true),
        ("What did we discuss about Python?", true),
        ("Find all coding related discussions", true),
        ("Show conversations from the last hour", true),
        ("What did the user ask about?", true),
    ];

    for (query, should_have_results) in queries {
        let request = MCPRequest {
            jsonrpc: "2.0".to_string(),
            id: Some(json!(1)),
            method: "tools/call".to_string(),
            params: Some(json!({
                "name": "query_conversations",
                "arguments": {
                    "query": query,
                    "limit": 10
                }
            })),
        };

        let response = server.handle_request(request).await;

        if let MCPResponse::ToolResult { content, .. } = response {
            let results: Value = serde_json::from_str(&content[0].text).unwrap();
            let conversations = results["conversations"].as_array().unwrap();

            if should_have_results {
                assert!(!conversations.is_empty(), "Query '{}' should return results", query);
            }

            println!("Query '{}' returned {} results", query, conversations.len());
        } else {
            panic!("Expected ToolResult response");
        }
    }
}

#[tokio::test]
async fn test_search_semantic() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "search_semantic",
            "arguments": {
                "query": "implementing data structures in Python",
                "content_types": ["transcript", "code", "ui_text"],
                "limit": 20
            }
        })),
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolResult { content, .. } = response {
        let results: Value = serde_json::from_str(&content[0].text).unwrap();
        assert!(results["results"].is_array());

        let search_results = results["results"].as_array().unwrap();
        assert!(!search_results.is_empty());

        // Check that results contain relevant content
        let has_binary_search = search_results.iter().any(|r| 
            r["content"].as_str().unwrap_or("").contains("binary search")
        );
        assert!(has_binary_search);
    } else {
        panic!("Expected ToolResult response");
    }
}

#[tokio::test]
async fn test_get_current_activity() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "get_current_activity",
            "arguments": {
                "window_minutes": 60
            }
        })),
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolResult { content, .. } = response {
        let results: Value = serde_json::from_str(&content[0].text).unwrap();

        assert!(results["current_app"].is_string());
        assert!(results["recent_activities"].is_array());
        assert!(results["detected_tasks"].is_array());

        // Should detect our test task
        let tasks = results["detected_tasks"].as_array().unwrap();
        assert!(!tasks.is_empty());
        assert_eq!(tasks[0]["type"], "CodingProblem");
    } else {
        panic!("Expected ToolResult response");
    }
}

#[tokio::test]
async fn test_query_multimodal_context() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "query_multimodal_context",
            "arguments": {
                "query": "What was happening when the user asked about Python code?",
                "time_window_minutes": 30
            }
        })),
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolResult { content, .. } = response {
        let results: Value = serde_json::from_str(&content[0].text).unwrap();

        assert!(results["contexts"].is_array());
        let contexts = results["contexts"].as_array().unwrap();

        // Should find correlation between transcript and visual data
        assert!(!contexts.is_empty());

        // Check that context includes both audio and visual events
        let first_context = &contexts[0];
        assert!(first_context["audio_events"].is_array());
        assert!(first_context["visual_events"].is_array());
        assert!(first_context["correlation_strength"].is_number());
    } else {
        panic!("Expected ToolResult response");
    }
}

#[tokio::test]
async fn test_find_assistance_opportunities() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "find_assistance_opportunities",
            "arguments": {
                "time_window_minutes": 60,
                "min_confidence": 0.7
            }
        })),
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolResult { content, .. } = response {
        let results: Value = serde_json::from_str(&content[0].text).unwrap();

        assert!(results["opportunities"].is_array());
        let opportunities = results["opportunities"].as_array().unwrap();

        // Should detect the coding problem as an assistance opportunity
        assert!(!opportunities.is_empty());

        let first_opp = &opportunities[0];
        assert!(first_opp["type"].is_string());
        assert!(first_opp["context"].is_object());
        assert!(first_opp["suggestions"].is_array());
    } else {
        panic!("Expected ToolResult response");
    }
}

#[tokio::test]
async fn test_correlate_audio_video_events() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "correlate_audio_video_events",
            "arguments": {
                "time_start": (Utc::now() - chrono::Duration::hours(1)).to_rfc3339(),
                "time_end": Utc::now().to_rfc3339(),
                "correlation_window_seconds": 60
            }
        })),
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolResult { content, .. } = response {
        let results: Value = serde_json::from_str(&content[0].text).unwrap();

        assert!(results["correlations"].is_array());
        assert!(results["summary"]["total_audio_events"].is_number());
        assert!(results["summary"]["total_video_events"].is_number());
        assert!(results["summary"]["strong_correlations"].is_number());
    } else {
        panic!("Expected ToolResult response");
    }
}

#[tokio::test]
async fn test_list_resources() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "resources/list".to_string(),
        params: None,
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ResourcesList { resources, .. } = response {
        assert!(!resources.is_empty());

        // Check key resources
        let resource_uris: Vec<&str> = resources.iter().map(|r| r.uri.as_str()).collect();
        assert!(resource_uris.contains(&"conversations://list"));
        assert!(resource_uris.contains(&"schema://database"));
        assert!(resource_uris.contains(&"multimodal_contexts://list"));
    } else {
        panic!("Expected ResourcesList response");
    }
}

#[tokio::test]
async fn test_complex_multimodal_query() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    // Add more complex test data
    let pool = &server.database.pool;

    // Add a compilation error scenario
    let error_frame_id = "error-frame-id";
    sqlx::query(
        r#"
        INSERT INTO hf_video_frames (id, timestamp_ms, session_id, frame_hash, change_score, active_app)
        VALUES (?, ?, 'session-2', 'error-frame', 0.95, 'Terminal')
        "#
    )
    .bind(error_frame_id)
    .bind(Utc::now().timestamp_millis() - 120000)
    .execute(pool).await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO hf_text_extractions (frame_id, word_text, confidence, bbox_x, bbox_y, bbox_width, bbox_height, text_type)
        VALUES 
            (?, 'SyntaxError:', 0.99, 100, 400, 100, 20, 'ErrorMessage'),
            (?, 'unexpected', 0.98, 200, 400, 80, 20, 'ErrorMessage'),
            (?, 'indent', 0.97, 280, 400, 60, 20, 'ErrorMessage')
        "#
    )
    .bind(error_frame_id)
    .bind(error_frame_id)
    .bind(error_frame_id)
    .execute(pool).await.unwrap();

    // Query for error context
    let request = MCPRequest {
        jsonrpc: "2.0".to_string(),
        id: Some(json!(1)),
        method: "tools/call".to_string(),
        params: Some(json!({
            "name": "query_multimodal_context",
            "arguments": {
                "query": "Find any compilation or syntax errors and their context",
                "time_window_minutes": 10
            }
        })),
    };

    let response = server.handle_request(request).await;

    if let MCPResponse::ToolResult { content, .. } = response {
        let results: Value = serde_json::from_str(&content[0].text).unwrap();
        let contexts = results["contexts"].as_array().unwrap();

        // Should find the error context
        let error_context = contexts.iter().find(|c| 
            c["visual_events"].as_array().unwrap().iter().any(|e|
                e["text_content"].as_str().unwrap_or("").contains("SyntaxError")
            )
        );

        assert!(error_context.is_some(), "Should find syntax error context");
    } else {
        panic!("Expected ToolResult response");
    }
}

#[tokio::test]
async fn test_performance_with_concurrent_requests() {
    let (server, _temp_dir) = setup_test_server().await.unwrap();

    // Spawn multiple concurrent requests
    let mut handles = vec![];

    for i in 0..10 {
        let server_clone = Arc::clone(&server);
        let handle = tokio::spawn(async move {
            let request = MCPRequest {
                jsonrpc: "2.0".to_string(),
                id: Some(json!(i)),
                method: "tools/call".to_string(),
                params: Some(json!({
                    "name": "query_conversations",
                    "arguments": {
                        "query": format!("test query {}", i),
                        "limit": 5
                    }
                })),
            };

            let start = std::time::Instant::now();
            let response = server_clone.handle_request(request).await;
            let elapsed = start.elapsed();

            (i, elapsed, response)
        });

        handles.push(handle);
    }

    // Wait for all requests to complete
    let mut total_time = std::time::Duration::from_secs(0);
    for handle in handles {
        let (id, elapsed, _response) = handle.await.unwrap();
        println!("Request {} completed in {:?}", id, elapsed);
        total_time += elapsed;
    }

    let avg_time = total_time / 10;
    println!("Average response time: {:?}", avg_time);

    // Should handle concurrent requests efficiently
    assert!(avg_time.as_millis() < 100);
}
