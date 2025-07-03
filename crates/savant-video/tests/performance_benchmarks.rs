use savant_video::*;
use savant_video::llm_provider::{LLMProvider, MockLLMProvider};
use savant_ocr::{ComprehensiveOCRProcessor, PreprocessingConfig};
use savant_vision::VisionAnalyzer;
use tempfile::TempDir;
use std::path::Path;
use std::time::{Duration, Instant};
use chrono::Utc;

async fn setup_performance_processor() -> IntegratedProcessor {
    let config = ProcessorConfig {
        enable_ocr: true,
        enable_vision: true,
        enable_real_time_analysis: true,
        enable_problem_detection: true,
        enable_auto_solutions: true,
        min_change_threshold: 0.05,
        processing_timeout_ms: 10000,
    };
    
    let mock_llm = MockLLMProvider::new();
    let temp_dir = TempDir::new().unwrap();
    let db_path = temp_dir.path().join("perf.db");
    
    let db_pool = sqlx::sqlite::SqlitePoolOptions::new()
        .max_connections(1)
        .connect(&format!("sqlite:{}", db_path.display()))
        .await
        .unwrap();
    
    sqlx::migrate!("../../savant-db/migrations")
        .run(&db_pool)
        .await
        .unwrap();
    
    let (processor, _) = IntegratedProcessor::new(
        config,
        LLMProvider::Mock(mock_llm),
        db_pool,
    )
    .await
    .unwrap();
    
    processor
}

#[tokio::test]
async fn benchmark_single_frame_processing() {
    let mut processor = setup_performance_processor().await;
    
    // Create test image
    let test_image = image::DynamicImage::new_rgb8(1920, 1080);
    let temp_dir = TempDir::new().unwrap();
    let image_path = temp_dir.path().join("benchmark.png");
    test_image.save(&image_path).unwrap();
    
    let frame = VideoFrame {
        id: "benchmark-frame".to_string(),
        timestamp: Utc::now(),
        file_path: image_path,
        resolution: (1920, 1080),
        file_size: 100000,
        image_hash: "benchmark-hash".to_string(),
        metadata: FrameMetadata {
            session_id: "benchmark-session".to_string(),
            display_id: None,
            active_application: Some("Visual Studio Code".to_string()),
            window_title: Some("main.py".to_string()),
            change_detected: true,
            ocr_text: None,
            enhanced_analysis: None,
            detected_applications: vec![],
            activity_classification: None,
            visual_context: None,
        },
    };
    
    // Benchmark processing
    let start = Instant::now();
    let result = processor.process_frame(&frame).await.unwrap();
    let elapsed = start.elapsed();
    
    println!("🔥 PERFORMANCE BENCHMARK RESULTS 🔥");
    println!("Frame Processing Time: {:?}", elapsed);
    println!("Processing Time (ms): {}", result.processing_time_ms);
    println!("Changes Detected: {}", result.changes_detected);
    println!("Text Extracted: {:?}", result.text_extracted);
    println!("Vision Analysis: {:?}", result.vision_analysis);
    println!("Problems Detected: {}", result.detected_problems.len());
    println!("Solutions Generated: {}", result.generated_solutions.len());
    
    // Performance assertions
    assert!(elapsed.as_millis() < 5000, "Processing should complete within 5 seconds");
    assert!(result.processing_time_ms < 5000, "Recorded processing time should be under 5 seconds");
}

#[tokio::test]
async fn benchmark_throughput() {
    let mut processor = setup_performance_processor().await;
    
    let frame_count = 10;
    let temp_dir = TempDir::new().unwrap();
    
    // Create test frames
    let mut frames = vec![];
    for i in 0..frame_count {
        let test_image = image::DynamicImage::new_rgb8(800, 600);
        let image_path = temp_dir.path().join(format!("frame_{}.png", i));
        test_image.save(&image_path).unwrap();
        
        frames.push(VideoFrame {
            id: format!("throughput-{}", i),
            timestamp: Utc::now(),
            file_path: image_path,
            resolution: (800, 600),
            file_size: 50000,
            image_hash: format!("hash-{}", i),
            metadata: Default::default(),
        });
    }
    
    // Benchmark throughput
    let start = Instant::now();
    let mut total_processing_time = 0u64;
    
    for frame in frames {
        let result = processor.process_frame(&frame).await.unwrap();
        total_processing_time += result.processing_time_ms;
    }
    
    let total_elapsed = start.elapsed();
    let avg_time_per_frame = total_elapsed.as_millis() / frame_count as u128;
    let frames_per_second = 1000.0 / avg_time_per_frame as f64;
    
    println!("🚀 THROUGHPUT BENCHMARK RESULTS 🚀");
    println!("Frames Processed: {}", frame_count);
    println!("Total Time: {:?}", total_elapsed);
    println!("Average Time per Frame: {}ms", avg_time_per_frame);
    println!("Frames per Second: {:.2}", frames_per_second);
    println!("Total Processing Time: {}ms", total_processing_time);
    println!("Processing Efficiency: {:.1}%", 
        (total_processing_time as f64 / total_elapsed.as_millis() as f64) * 100.0);
    
    // Throughput assertions
    assert!(frames_per_second > 0.1, "Should process at least 0.1 frames per second");
    assert!(avg_time_per_frame < 10000, "Average processing time should be under 10 seconds");
}

#[tokio::test]
async fn benchmark_with_real_screenshots() {
    let mut processor = setup_performance_processor().await;
    
    // Load test screenshots
    let test_data_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap()
        .parent().unwrap()
        .join("test-data/screenshots");
    
    let screenshots = vec![
        "twosum.png",
        "hackerrank_hard_01.png", 
        "getcracked_medium_01.png",
    ];
    
    let mut benchmark_results = vec![];
    
    for screenshot in screenshots {
        let screenshot_path = test_data_dir.join(screenshot);
        if !screenshot_path.exists() {
            println!("⚠️  Skipping missing screenshot: {}", screenshot);
            continue;
        }
        
        let frame = VideoFrame {
            id: format!("bench-{}", screenshot),
            timestamp: Utc::now(),
            file_path: screenshot_path,
            resolution: (1920, 1080),
            file_size: 200000,
            image_hash: format!("hash-{}", screenshot),
            metadata: FrameMetadata {
                session_id: "benchmark-session".to_string(),
                display_id: None,
                active_application: Some("Browser".to_string()),
                window_title: Some("Coding Challenge".to_string()),
                change_detected: true,
                ocr_text: None,
                enhanced_analysis: None,
                detected_applications: vec![],
                activity_classification: None,
                visual_context: None,
            },
        };
        
        // Benchmark this specific screenshot
        let start = Instant::now();
        let result = processor.process_frame(&frame).await.unwrap();
        let elapsed = start.elapsed();
        
        benchmark_results.push((
            screenshot,
            elapsed,
            result.processing_time_ms,
            result.detected_problems.len(),
            result.generated_solutions.len(),
        ));
        
        println!("📸 {} - Processing Time: {:?}, Problems: {}, Solutions: {}", 
            screenshot, elapsed, result.detected_problems.len(), result.generated_solutions.len());
    }
    
    println!("\n🏆 REAL SCREENSHOT BENCHMARK SUMMARY 🏆");
    for (name, elapsed, processing_ms, problems, solutions) in &benchmark_results {
        println!("{}: {:?} ({}ms) -> {} problems, {} solutions", 
            name, elapsed, processing_ms, problems, solutions);
    }
    
    if !benchmark_results.is_empty() {
        let avg_time: Duration = benchmark_results
            .iter()
            .map(|(_, elapsed, _, _, _)| *elapsed)
            .sum::<Duration>() / benchmark_results.len() as u32;
        
        let total_problems: usize = benchmark_results
            .iter()
            .map(|(_, _, _, problems, _)| *problems)
            .sum();
        
        let total_solutions: usize = benchmark_results
            .iter()
            .map(|(_, _, _, _, solutions)| *solutions)
            .sum();
        
        println!("\n📊 AGGREGATE METRICS:");
        println!("Average Processing Time: {:?}", avg_time);
        println!("Total Problems Detected: {}", total_problems);
        println!("Total Solutions Generated: {}", total_solutions);
        println!("Problem Detection Rate: {:.1}%", 
            (total_problems as f64 / benchmark_results.len() as f64) * 100.0);
        
        // Performance requirements
        assert!(avg_time.as_secs() < 10, "Average processing should be under 10 seconds");
        assert!(total_problems > 0, "Should detect at least one problem across all screenshots");
    }
}

#[tokio::test]
async fn benchmark_memory_usage() {
    use std::alloc::{GlobalAlloc, Layout, System};
    use std::sync::atomic::{AtomicUsize, Ordering};
    
    static ALLOCATED: AtomicUsize = AtomicUsize::new(0);
    
    struct TrackingAllocator;
    
    unsafe impl GlobalAlloc for TrackingAllocator {
        unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
            let ret = System.alloc(layout);
            if !ret.is_null() {
                ALLOCATED.fetch_add(layout.size(), Ordering::SeqCst);
            }
            ret
        }
        
        unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
            System.dealloc(ptr, layout);
            ALLOCATED.fetch_sub(layout.size(), Ordering::SeqCst);
        }
    }
    
    #[global_allocator]
    static GLOBAL: TrackingAllocator = TrackingAllocator;
    
    let initial_memory = ALLOCATED.load(Ordering::SeqCst);
    
    let mut processor = setup_performance_processor().await;
    
    // Create multiple frames to test memory usage
    let temp_dir = TempDir::new().unwrap();
    for i in 0..5 {
        let test_image = image::DynamicImage::new_rgb8(1920, 1080);
        let image_path = temp_dir.path().join(format!("memory_test_{}.png", i));
        test_image.save(&image_path).unwrap();
        
        let frame = VideoFrame {
            id: format!("memory-test-{}", i),
            timestamp: Utc::now(),
            file_path: image_path,
            resolution: (1920, 1080),
            file_size: 100000,
            image_hash: format!("hash-{}", i),
            metadata: Default::default(),
        };
        
        let _result = processor.process_frame(&frame).await.unwrap();
        
        let current_memory = ALLOCATED.load(Ordering::SeqCst);
        let memory_used = current_memory.saturating_sub(initial_memory);
        
        println!("Frame {}: Memory used: {} KB", i, memory_used / 1024);
    }
    
    let final_memory = ALLOCATED.load(Ordering::SeqCst);
    let total_memory_used = final_memory.saturating_sub(initial_memory);
    
    println!("\n💾 MEMORY USAGE BENCHMARK:");
    println!("Initial Memory: {} KB", initial_memory / 1024);
    println!("Final Memory: {} KB", final_memory / 1024);
    println!("Total Memory Used: {} KB", total_memory_used / 1024);
    println!("Average Memory per Frame: {} KB", total_memory_used / (5 * 1024));
    
    // Memory usage should be reasonable
    assert!(total_memory_used < 500 * 1024 * 1024, "Total memory usage should be under 500MB");
}