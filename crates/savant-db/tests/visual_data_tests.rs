use anyhow::Result;
use savant_db::visual_data::{VisualDataManager, VideoQuery};
use savant_ocr::{ComprehensiveOCRResult, TextType, WordData, BoundingBox};
use sqlx::sqlite::SqlitePoolOptions;
use tempfile::TempDir;
use chrono::{Utc, TimeZone};
use std::str::FromStr;
use serde_json;
use std::path::PathBuf;

// Mock types to replace savant-video dependencies
#[derive(Clone)]
struct VideoFrame {
    id: String,
    timestamp: chrono::DateTime<Utc>,
    file_path: PathBuf,
    resolution: (u32, u32),
    file_size: u64,
    image_hash: String,
    metadata: FrameMetadata,
}

#[derive(Clone)]
struct FrameMetadata {
    session_id: String,
    display_id: Option<String>,
    active_application: Option<String>,
    window_title: Option<String>,
    change_detected: bool,
    ocr_text: Option<String>,
    enhanced_analysis: Option<serde_json::Value>,
    detected_applications: Vec<String>,
    activity_classification: Option<String>,
    visual_context: Option<serde_json::Value>,
}

async fn setup_test_db() -> Result<(VisualDataManager, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db_path = temp_dir.path().join("test.db");

    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}", db_path.display()))
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    let manager = VisualDataManager::new(pool);
    Ok((manager, temp_dir))
}

#[tokio::test]
async fn test_store_and_retrieve_frame() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    // Store a frame
    let frame = savant_video::VideoFrame {
        id: "test-frame-1".to_string(),
        timestamp: Utc::now(),
        file_path: "/tmp/frame.png".into(),
        resolution: (1920, 1080),
        file_size: 1000,
        image_hash: "abc123".to_string(),
        metadata: savant_video::FrameMetadata {
            session_id: "test-session".to_string(),
            display_id: None,
            active_application: Some("Visual Studio Code".to_string()),
            window_title: None,
            change_detected: false,
            ocr_text: None,
            enhanced_analysis: None,
            detected_applications: Vec::new(),
            activity_classification: None,
            visual_context: None,
        },
    };

    manager.store_frame(&frame).await.unwrap();

    // Since `get_frames_in_range` is not directly exposed, we'll query using `query_frames`
    let query = savant_db::visual_data::VideoQuery {
        session_id: Some("test-session".to_string()),
        ..Default::default()
    };
    let frames = manager.query_frames(&query).await.unwrap();

    assert_eq!(frames.len(), 1);
    assert_eq!(frames[0]["image_hash"], "abc123");
    assert_eq!(frames[0]["active_application"], "Visual Studio Code");
}

#[tokio::test]
async fn test_store_text_extractions() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    // First store a frame
    let frame_id = "test-frame-1";
    let timestamp_ms = Utc::now().timestamp_millis();

    let frame = HighFrequencyFrame {
        timestamp_ms,
        session_id: "test-session".to_string(),
        frame_hash: frame_id.to_string(),
        change_score: 0.9,
        file_path: None,
        screen_resolution: None,
        active_app: None,
        processing_flags: 0,
    };

    manager.store_frame(&frame).await.unwrap();

    // Store text extractions
    let extractions = vec![
        TextExtraction {
            frame_id: frame_id.to_string(),
            word_text: "Hello".to_string(),
            confidence: 0.95,
            bbox_x: 100,
            bbox_y: 200,
            bbox_width: 50,
            bbox_height: 20,
            font_size_estimate: Some(14.0),
            text_type: Some("UIElement".to_string()),
            line_id: 0,
            paragraph_id: 0,
        },
        TextExtraction {
            frame_id: frame_id.to_string(),
            word_text: "World".to_string(),
            confidence: 0.92,
            bbox_x: 155,
            bbox_y: 200,
            bbox_width: 50,
            bbox_height: 20,
            font_size_estimate: Some(14.0),
            text_type: Some("UIElement".to_string()),
            line_id: 0,
            paragraph_id: 0,
        },
    ];

    for extraction in &extractions {
        manager.store_text_extraction(extraction).await.unwrap();
    }

    // Query text extractions
    let retrieved = manager.get_text_in_region(
        timestamp_ms - 1000,
        timestamp_ms + 1000,
        50,  // x_min
        150, // y_min
        300, // x_max
        250, // y_max
    ).await.unwrap();

    assert_eq!(retrieved.len(), 2);
    assert!(retrieved.iter().any(|t| t.word_text == "Hello"));
    assert!(retrieved.iter().any(|t| t.word_text == "World"));
}

#[tokio::test]
async fn test_search_text_content() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    // Store frames with text
    let test_data = vec![
        ("frame1", "def calculate_sum(a, b):", "CodeSnippet"),
        ("frame2", "return a + b", "CodeSnippet"),
        ("frame3", "Error: undefined variable", "ErrorMessage"),
        ("frame4", "Click here to continue", "UIElement"),
    ];

    let base_time = Utc::now().timestamp_millis();

    for (i, (frame_id, text, text_type)) in test_data.iter().enumerate() {
        // Store frame
        let frame = HighFrequencyFrame {
            timestamp_ms: base_time + (i as i64 * 1000),
            session_id: "test-session".to_string(),
            frame_hash: frame_id.to_string(),
            change_score: 0.5,
            file_path: None,
            screen_resolution: None,
            active_app: None,
            processing_flags: 0,
        };
        manager.store_frame(&frame).await.unwrap();

        // Store text
        let extraction = TextExtraction {
            frame_id: frame_id.to_string(),
            word_text: text.to_string(),
            confidence: 0.9,
            bbox_x: 100,
            bbox_y: 100 + (i as i32 * 50),
            bbox_width: 200,
            bbox_height: 30,
            font_size_estimate: Some(12.0),
            text_type: Some(text_type.to_string()),
            line_id: 0,
            paragraph_id: 0,
        };
        manager.store_text_extraction(&extraction).await.unwrap();
    }

    // Search for code-related text
    let code_results = manager.search_text_content(
        "calculate_sum",
        base_time - 1000,
        base_time + 10000,
    ).await.unwrap();

    assert_eq!(code_results.len(), 1);
    assert!(code_results[0].word_text.contains("calculate_sum"));

    // Search for error messages
    let error_results = manager.search_text_content(
        "Error",
        base_time - 1000,
        base_time + 10000,
    ).await.unwrap();

    assert_eq!(error_results.len(), 1);
    assert_eq!(error_results[0].text_type, Some("ErrorMessage".to_string()));
}

#[tokio::test]
async fn test_get_activity_summary() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    // Store frames with different applications
    let apps = vec![
        ("Visual Studio Code", 10),
        ("Chrome", 5),
        ("Terminal", 3),
    ];

    let base_time = Utc::now().timestamp_millis();
    let test_session_id = "activity-test-session";

    for (app_name, count) in apps {
        for i in 0..count {
            let frame = savant_video::VideoFrame {
                id: format!("{}-{}-{}", app_name, count, i),
                timestamp: chrono::Utc.timestamp_millis(base_time + (i as i64 * 100)),
                file_path: "/tmp/app_usage_frame.png".into(),
                resolution: (1920, 1080),
                file_size: 1000,
                image_hash: format!("{}-{}-{}", app_name, count, i),
                metadata: savant_video::FrameMetadata {
                    session_id: test_session_id.to_string(),
                    display_id: None,
                    active_application: Some(app_name.to_string()),
                    window_title: None,
                    change_detected: false,
                    ocr_text: None,
                    enhanced_analysis: None,
                    detected_applications: Vec::new(),
                    activity_classification: None,
                    visual_context: None,
                },
            };
            manager.store_frame(&frame).await.unwrap();
        }
    }

    // Get activity summary
    let summary = manager.get_activity_summary(
        base_time - 1000,
        base_time + 20000,
    ).await.unwrap();

    // Verify app usage statistics
    assert_eq!(summary.len(), 3);

    let vscode_summary = summary.iter().find(|s| s.app_name == "Visual Studio Code").unwrap();
    assert_eq!(vscode_summary.frame_count, 10);

    let chrome_summary = summary.iter().find(|s| s.app_name == "Chrome").unwrap();
    assert_eq!(chrome_summary.frame_count, 5);
}

#[tokio::test]
async fn test_detected_tasks_storage() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    let frame_id = "task-frame-1";
    let task = savant_db::visual_data::DetectedTask {
        frame_id: frame_id.to_string(),
        task_type: "CodingProblem".to_string(),
        confidence: 0.92,
        description: "Implement two sum algorithm".to_string(),
        evidence_text: r#"{"title": "Two Sum", "platform": "LeetCode"}"#.to_string(),
        bounding_regions: Some(r#"{"x": 100, "y": 200, "width": 800, "height": 600}"#.to_string()),
        assistance_suggestions: r#"["Use hash map for O(n) solution", "Consider edge cases"]"#.to_string(),
    };

    manager.store_detected_task(&task).await.unwrap();

    // Retrieve tasks
    let tasks = manager.get_recent_tasks(
        Utc::now().timestamp_millis() - 60000,
        Utc::now().timestamp_millis() + 60000,
        10,
    ).await.unwrap();

    assert_eq!(tasks.len(), 1);
    assert_eq!(tasks[0].task_type, "CodingProblem");
    assert!(tasks[0].description.contains("two sum"));
}

#[tokio::test]
async fn test_complex_query_scenarios() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    // Simulate a coding session
    let base_time = Utc::now().timestamp_millis();
    let session_id = "coding-session-1";

    // Frame 1: Looking at LeetCode problem
    let frame1 = savant_video::VideoFrame {
        id: "filter_frame_1".to_string(),
        timestamp: chrono::Utc.timestamp_millis(base_time),
        file_path: "/tmp/filter_frame_1.png".into(),
        resolution: (1920, 1080),
        file_size: 1000,
        image_hash: "filter_frame_1".to_string(),
        metadata: savant_video::FrameMetadata {
            session_id: session_id.to_string(),
            display_id: None,
            active_application: Some("Visual Studio Code".to_string()),
            window_title: None,
            change_detected: false,
            ocr_text: None,
            enhanced_analysis: None,
            detected_applications: Vec::new(),
            activity_classification: None,
            visual_context: None,
        },
    };
    manager.store_frame(&frame1).await.unwrap();
    manager.store_code_snippet(&serde_json::json!({
        "frame_id": frame1.id.clone(),
        "programming_language": "Python".to_string(),
        "code_content": "print('hello')".to_string(),
        "complexity_score": 0.1,
        "context": "".to_string(),
    })).await.unwrap();
    manager.store_interaction_opportunity(&serde_json::json!({
        "frame_id": frame1.id.clone(),
        "opportunity_type": "Test".to_string(),
        "description": "Test".to_string(),
        "confidence": 0.5,
        "suggested_action": "Test".to_string(),
        "context_info": "Test".to_string(),
        "urgency": "High".to_string(),
    })).await.unwrap();
    manager.store_ocr_content(&frame1.id, &serde_json::json!({"text": "Python code here"})).await.unwrap();

    // Test complex queries

    // 1. Find all code snippets in the last minute
    let code_snippets = sqlx::query!(
        r#"
        SELECT DISTINCT word_text, bbox_x, bbox_y
        FROM hf_text_extractions t
        JOIN hf_video_frames f ON t.frame_id = f.frame_hash
        WHERE f.timestamp_ms > ?1
          AND t.text_type = 'CodeSnippet'
        ORDER BY f.timestamp_ms, t.bbox_y, t.bbox_x
        "#,
        base_time - 1000
    )
    .fetch_all(manager.pool())
    .await
    .unwrap();

    assert!(code_snippets.len() >= 8); // All code words
    assert!(code_snippets.iter().any(|r| r.word_text == "twoSum"));

    // 2. Find transitions between applications
    let app_transitions = sqlx::query!(
        r#"
        SELECT active_app, timestamp_ms
        FROM hf_video_frames
        WHERE session_id = ?1
        ORDER BY timestamp_ms
        "#,
        session_id
    )
    .fetch_all(manager.pool())
    .await
    .unwrap();

    assert_eq!(app_transitions.len(), 2);
    assert_eq!(app_transitions[0].active_app, Some("Chrome".to_string()));
    assert_eq!(app_transitions[1].active_app, Some("Visual Studio Code".to_string()));

    // 3. Correlate problem description with code implementation
    let problem_to_code_correlation = sqlx::query!(
        r#"
        WITH problem_text AS (
            SELECT GROUP_CONCAT(t.word_text, ' ') as problem_description
            FROM hf_text_extractions t
            JOIN hf_video_frames f ON t.frame_id = f.frame_hash
            WHERE f.active_app = 'Chrome'
              AND t.text_type = 'DocumentContent'
        ),
        code_text AS (
            SELECT GROUP_CONCAT(t.word_text, ' ') as code_implementation
            FROM hf_text_extractions t
            JOIN hf_video_frames f ON t.frame_id = f.frame_hash
            WHERE f.active_app = 'Visual Studio Code'
              AND t.text_type = 'CodeSnippet'
        )
        SELECT 
            (SELECT problem_description FROM problem_text) as problem,
            (SELECT code_implementation FROM code_text) as code
        "#
    )
    .fetch_one(manager.pool())
    .await
    .unwrap();

    assert!(problem_to_code_correlation.problem.unwrap().contains("array of integers"));
    assert!(problem_to_code_correlation.code.unwrap().contains("twoSum"));
}

#[tokio::test]
async fn test_performance_with_large_dataset() {
    let (manager, _temp_dir) = setup_test_db().await.unwrap();

    let base_time = Utc::now().timestamp_millis();
    let num_frames = 1000;
    let words_per_frame = 50;

    // Insert many frames
    let start = std::time::Instant::now();

    for i in 0..num_frames {
        let frame = HighFrequencyFrame {
            timestamp_ms: base_time + (i * 500), // 500ms intervals
            session_id: "perf-test".to_string(),
            frame_hash: format!("frame-{}", i),
            change_score: 0.5,
            file_path: None,
            screen_resolution: Some("1920x1080".to_string()),
            active_app: Some("TestApp".to_string()),
            processing_flags: 0,
        };
        manager.store_frame(&frame).await.unwrap();

        // Add some text extractions
        for j in 0..words_per_frame {
            let extraction = TextExtraction {
                frame_id: format!("frame-{}", i),
                word_text: format!("word{}_{}", i, j),
                confidence: 0.9,
                bbox_x: (j * 50) as i32,
                bbox_y: 100,
                bbox_width: 45,
                bbox_height: 20,
                font_size_estimate: Some(12.0),
                text_type: Some("UIElement".to_string()),
                line_id: 0,
                paragraph_id: 0,
            };
            manager.store_text_extraction(&extraction).await.unwrap();
        }
    }

    let insert_time = start.elapsed();
    println!("Inserted {} frames with {} words each in {:?}", 
        num_frames, words_per_frame, insert_time);

    // Test query performance
    let query_start = std::time::Instant::now();

    let results = manager.get_frames_in_range(
        base_time,
        base_time + (num_frames * 1000),
        100, // Limit to 100 results
    ).await.unwrap();

    let query_time = query_start.elapsed();
    println!("Query returned {} results in {:?}", results.len(), query_time);

    assert_eq!(results.len(), 100);
    assert!(query_time.as_millis() < 100); // Should be very fast with indexes
}
