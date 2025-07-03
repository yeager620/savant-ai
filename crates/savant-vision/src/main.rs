use anyhow::Result;
use clap::{Parser, Subcommand};
use image::ImageReader;
use savant_vision::{VisionAnalyzer, ScreenAnalysis, VisionConfig};
use serde_json;
use std::path::PathBuf;
use tokio;

#[derive(Parser)]
#[command(name = "savant-vision")]
#[command(about = "Computer vision analysis for screen content")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze a screenshot image
    Analyze {
        /// Path to the image file
        #[arg(short, long)]
        input: PathBuf,

        /// Output format (json, summary, detailed)
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Enable detailed application detection
        #[arg(long)]
        detect_apps: bool,

        /// Enable activity classification
        #[arg(long)]
        classify_activity: bool,

        /// Enable UI element detection
        #[arg(long)]
        detect_ui: bool,
    },

    /// Process images from stdin (for pipeline usage)
    Process {
        /// Output format
        #[arg(short, long, default_value = "json")]
        format: String,

        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },

    /// Test vision analysis capabilities
    Test {
        /// Test image path
        #[arg(short, long)]
        input: Option<PathBuf>,
    },

    /// Benchmark vision processing performance
    Benchmark {
        /// Test image path
        #[arg(short, long)]
        input: PathBuf,

        /// Number of iterations
        #[arg(short, long, default_value = "10")]
        iterations: u32,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let cli = Cli::parse();

    match cli.command {
        Commands::Analyze {
            input,
            format,
            detect_apps,
            classify_activity,
            detect_ui,
        } => {
            analyze_image(input, format, detect_apps, classify_activity, detect_ui).await?;
        }
        Commands::Process { format, config } => {
            process_from_stdin(format, config).await?;
        }
        Commands::Test { input } => {
            test_vision_analysis(input).await?;
        }
        Commands::Benchmark { input, iterations } => {
            benchmark_analysis(input, iterations).await?;
        }
    }

    Ok(())
}

async fn analyze_image(
    input: PathBuf,
    format: String,
    _detect_apps: bool,
    _classify_activity: bool,
    _detect_ui: bool,
) -> Result<()> {
    // Load image
    let image = ImageReader::open(&input)?.decode()?;

    // Create vision analyzer
    let analyzer = VisionAnalyzer::new(VisionConfig::default())?;
    let analysis = analyzer.analyze_screen(&image).await?;

    // Output results based on format
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&analysis)?);
        }
        "summary" => {
            print_analysis_summary(&analysis);
        }
        "detailed" => {
            print_detailed_analysis(&analysis);
        }
        _ => {
            anyhow::bail!("Unsupported format: {}", format);
        }
    }

    Ok(())
}

fn print_analysis_summary(analysis: &ScreenAnalysis) {
    println!("=== Screen Analysis Summary ===");
    println!("Timestamp: {}", analysis.timestamp);
    println!("Processing Time: {}ms", analysis.processing_time_ms);
    println!("Image Size: {}x{}", analysis.image_metadata.width, analysis.image_metadata.height);

    println!("\n=== Detected Applications ===");
    for app in &analysis.app_context.detected_applications {
        println!("- {:?} (confidence: {:.2})", app.app_type, app.confidence);
    }

    println!("\n=== Primary Activity ===");
    println!("{:?} (confidence: {:.2})", 
        analysis.activity_classification.primary_activity, 
        analysis.activity_classification.confidence);

    if !analysis.activity_classification.secondary_activities.is_empty() {
        println!("\n=== Secondary Activities ===");
        for activity in &analysis.activity_classification.secondary_activities {
            println!("- {:?}", activity);
        }
    }

    println!("\n=== Visual Elements ===");
    println!("Total elements detected: {}", analysis.visual_elements.len());

    let mut element_counts = std::collections::HashMap::new();
    for element in &analysis.visual_elements {
        *element_counts.entry(format!("{:?}", element.element_type)).or_insert(0) += 1;
    }

    for (element_type, count) in element_counts {
        println!("- {}: {}", element_type, count);
    }
}

fn print_detailed_analysis(analysis: &ScreenAnalysis) {
    print_analysis_summary(analysis);

    println!("\n=== Detailed Visual Context ===");
    println!("Theme: {} mode", if analysis.visual_context.theme_info.is_dark_mode { "Dark" } else { "Light" });
    println!("Background: {}", analysis.visual_context.theme_info.background_color);
    println!("Contrast Ratio: {:.1}", analysis.visual_context.theme_info.contrast_ratio);

    println!("\n=== Layout Analysis ===");
    println!("Layout Type: {:?}", analysis.visual_context.layout_analysis.layout_type);
    println!("Header Present: {}", analysis.visual_context.layout_analysis.header_present);
    println!("Sidebar Present: {}", analysis.visual_context.layout_analysis.sidebar_present);
    println!("Footer Present: {}", analysis.visual_context.layout_analysis.footer_present);

    if !analysis.visual_context.attention_areas.is_empty() {
        println!("\n=== High Attention Areas ===");
        for (i, area) in analysis.visual_context.attention_areas.iter().enumerate() {
            println!("{}. Score: {:.2}, Reason: {:?}", 
                i + 1, area.attention_score, area.reason);
        }
    }

    if !analysis.visual_context.interaction_elements.is_empty() {
        println!("\n=== Interactive Elements ===");
        for element in &analysis.visual_context.interaction_elements {
            println!("- {:?} (accessibility: {:.2})", 
                element.element_type, element.accessibility_score);
        }
    }

    println!("\n=== Context Indicators ===");
    for indicator in &analysis.activity_classification.context_indicators {
        println!("- {:?}: {} (confidence: {:.2})", 
            indicator.indicator_type, indicator.value, indicator.confidence);
    }

    if !analysis.activity_classification.evidence.is_empty() {
        println!("\n=== Evidence ===");
        for evidence in &analysis.activity_classification.evidence {
            println!("- {:?}: {} (confidence: {:.2}, weight: {:.2})", 
                evidence.evidence_type, evidence.description, 
                evidence.confidence, evidence.weight);
        }
    }
}

async fn process_from_stdin(_format: String, _config: Option<PathBuf>) -> Result<()> {
    // Placeholder for stdin processing
    anyhow::bail!("Stdin processing not yet implemented");
}

async fn test_vision_analysis(input: Option<PathBuf>) -> Result<()> {
    println!("Testing vision analysis capabilities...");

    let test_image = if let Some(path) = input {
        ImageReader::open(path)?.decode()?
    } else {
        // Create a simple test image
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::from_fn(400, 300, |x, y| {
            if x > 50 && x < 150 && y > 50 && y < 100 {
                Rgb([100, 150, 255]) // Blue rectangle (button-like)
            } else if x > 200 && x < 350 && y > 100 && y < 250 {
                Rgb([50, 50, 50]) // Dark rectangle (terminal-like)
            } else {
                Rgb([240, 240, 240]) // Light background
            }
        });
        image::DynamicImage::ImageRgb8(img)
    };

    let analyzer = VisionAnalyzer::new(VisionConfig::default())?;

    let start_time = std::time::Instant::now();
    let analysis = analyzer.analyze_screen(&test_image).await?;
    let processing_time = start_time.elapsed();

    println!("✓ Vision analysis completed in {:?}", processing_time);
    println!("✓ Detected {} visual elements", analysis.visual_elements.len());
    println!("✓ Identified {} applications", analysis.app_context.detected_applications.len());
    println!("✓ Primary activity: {:?}", analysis.activity_classification.primary_activity);
    println!("✓ Processing time: {}ms", analysis.processing_time_ms);

    if !analysis.visual_context.dominant_colors.is_empty() {
        println!("✓ Dominant colors: {:?}", analysis.visual_context.dominant_colors);
    }

    println!("✓ Theme: {} mode", 
        if analysis.visual_context.theme_info.is_dark_mode { "Dark" } else { "Light" });

    Ok(())
}

async fn benchmark_analysis(input: PathBuf, iterations: u32) -> Result<()> {
    println!("Benchmarking vision analysis performance...");

    let image = ImageReader::open(&input)?.decode()?;
    let analyzer = VisionAnalyzer::new(VisionConfig::default())?;

    let mut total_time = std::time::Duration::ZERO;
    let mut successful_runs = 0;

    for i in 1..=iterations {
        print!("Iteration {}/{}... ", i, iterations);

        let start_time = std::time::Instant::now();
        match analyzer.analyze_screen(&image).await {
            Ok(_) => {
                let elapsed = start_time.elapsed();
                total_time += elapsed;
                successful_runs += 1;
                println!("✓ {}ms", elapsed.as_millis());
            }
            Err(e) => {
                println!("✗ Error: {}", e);
            }
        }
    }

    if successful_runs > 0 {
        let average_time = total_time / successful_runs;
        println!("\n=== Benchmark Results ===");
        println!("Successful runs: {}/{}", successful_runs, iterations);
        println!("Average processing time: {:?}", average_time);
        println!("Images per second: {:.2}", 1000.0 / average_time.as_millis() as f64);
        println!("Total time: {:?}", total_time);
    } else {
        println!("No successful runs completed.");
    }

    Ok(())
}
