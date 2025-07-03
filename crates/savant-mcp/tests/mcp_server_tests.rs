use anyhow::Result;
use serde_json::{json, Value};
use savant_mcp::{MCPServer, MCPRequest, MCPResponse};
use sqlx::sqlite::SqlitePoolOptions;
use tempfile::TempDir;
use chrono::Utc;

async fn setup_test_server() -> Result<(MCPServer, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}", db_path.display()))
        .await?;

    // Run migrations
    sqlx::migrate!("../savant-db/migrations")
        .run(&pool)
        .await?;

    // Insert test data
    sqlx::query!(
        r#"
        INSERT INTO conversations (id, started_at, title)
        VALUES ('conv-1', datetime('now', '-1 hour'), 'Test Conversation')
        "#
    ).execute(&pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO transcripts (conversation_id, timestamp, speaker_id, text, confidence)
        VALUES 
            ('conv-1', datetime('now', '-30 minutes'), 'user', 'How do I implement a binary search tree?', 0.95),
            ('conv-1', datetime('now', '-29 minutes'), 'assistant', 'To implement a binary search tree, you need to create a Node class...', 0.98),
            ('conv-1', datetime('now', '-15 minutes'), 'user', 'Can you show me the code in Python?', 0.93),
            ('conv-1', datetime('now', '-14 minutes'), 'assistant', 'Here is a Python implementation of a binary search tree...', 0.97)
        "#
    ).execute(&pool).await?;

    // Add visual data
    sqlx::query!(
        r#"
        INSERT INTO hf_video_frames (timestamp_ms, session_id, frame_hash, change_score, active_app)
        VALUES 
            (?1, 'session-1', 'frame-1', 0.9, 'Visual Studio Code'),
            (?2, 'session-1', 'frame-2', 0.85, 'Chrome'),
            (?3, 'session-1', 'frame-3', 0.7, 'Terminal')
        "#,
        Utc::now().timestamp_millis() - 1800000, // 30 minutes ago
        Utc::now().timestamp_millis() - 900000,  // 15 minutes ago
        Utc::now().timestamp_millis() - 300000,  // 5 minutes ago
    ).execute(&pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO hf_text_extractions (frame_id, word_text, confidence, bbox_x, bbox_y, bbox_width, bbox_height, text_type)
        VALUES 
            ('frame-1', 'class', 0.98, 100, 200, 50, 20, 'CodeSnippet'),
            ('frame-1', 'BinarySearchTree', 0.97, 150, 200, 150, 20, 'CodeSnippet'),
            ('frame-2', 'LeetCode', 0.99, 500, 100, 100, 30, 'UIElement'),
            ('frame-3', 'python', 0.95, 200, 300, 60, 18, 'CommandLine')
        "#
    ).execute(&pool).await?;

    sqlx::query!(
        r#"
        INSERT INTO hf_detected_tasks (frame_id, task_type, confidence, description, evidence_text)
        VALUES 
            ('frame-1', 'CodingProblem', 0.92, 'Implementing binary search tree', '{"language": "Python", "complexity": "Medium"}')
        "#
    ).execute(&pool).await?;

    let database = Arc::new(savant_db::TranscriptDatabase::new(pool));
    let server = MCPServer::new(database, None).await?;
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

    let response = server.handle_request(request).await.unwrap();

    if let MCPResponse::ToolsList { tools, .. } = response {
        assert!(!tools.is_empty());

        // Check that key tools are present
        let tool_names: Vec<&str> = tools.iter().map(|t| t.name.as_str()).collect();
        assert!(tool_names.contains(&"query_conversations"));
        assert!(tool_names.contains(&"search_semantic"));
        assert!(tool_names.contains(&"get_current_activity"));
        assert!(tool_names.contains(&"query_multimodal_context"));
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

        let response = server.handle_request(request).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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
    let pool = &server.db_pool;

    // Add a compilation error scenario
    sqlx::query!(
        r#"
        INSERT INTO hf_video_frames (timestamp_ms, session_id, frame_hash, change_score, active_app)
        VALUES (?1, 'session-2', 'error-frame', 0.95, 'Terminal')
        "#,
        Utc::now().timestamp_millis() - 120000, // 2 minutes ago
    ).execute(pool).await.unwrap();

    sqlx::query!(
        r#"
        INSERT INTO hf_text_extractions (frame_id, word_text, confidence, bbox_x, bbox_y, bbox_width, bbox_height, text_type)
        VALUES 
            ('error-frame', 'SyntaxError:', 0.99, 100, 400, 100, 20, 'ErrorMessage'),
            ('error-frame', 'unexpected', 0.98, 200, 400, 80, 20, 'ErrorMessage'),
            ('error-frame', 'indent', 0.97, 280, 400, 60, 20, 'ErrorMessage')
        "#
    ).execute(pool).await.unwrap();

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

    let response = server.handle_request(request).await.unwrap();

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
        let server_clone = server.clone();
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
            let response = server_clone.handle_request(request).await.unwrap();
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
