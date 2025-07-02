use anyhow::Result;
use clap::{Parser, Subcommand};
use image::ImageReader;
use savant_ocr::{OCRConfig, OCRProcessor};
use serde_json;
use std::path::PathBuf;
use tokio;

#[derive(Parser)]
#[command(name = "savant-ocr")]
#[command(about = "Text extraction and analysis from images")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Extract text from an image file
    Extract {
        /// Path to the image file
        #[arg(short, long)]
        input: PathBuf,
        
        /// Output format (json, text, structured)
        #[arg(short, long, default_value = "json")]
        format: String,
        
        /// OCR engine to use
        #[arg(short, long, default_value = "tesseract")]
        engine: String,
        
        /// Languages to detect (comma-separated)
        #[arg(short, long, default_value = "eng")]
        languages: String,
        
        /// Minimum confidence threshold
        #[arg(short, long, default_value = "0.5")]
        confidence: f32,
        
        /// Enable text classification
        #[arg(long)]
        classify: bool,
        
        /// Enable structure analysis
        #[arg(long)]
        analyze: bool,
        
        /// Use fast processing mode (less accurate but much faster)
        #[arg(long)]
        fast: bool,
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
    
    /// Test OCR capabilities
    Test {
        /// Test image path
        #[arg(short, long)]
        input: Option<PathBuf>,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Extract {
            input,
            format,
            engine,
            languages,
            confidence,
            classify,
            analyze,
            fast,
        } => {
            extract_text(input, format, engine, languages, confidence, classify, analyze, fast).await?;
        }
        Commands::Process { format, config } => {
            process_from_stdin(format, config).await?;
        }
        Commands::Test { input } => {
            test_ocr(input).await?;
        }
    }
    
    Ok(())
}

async fn extract_text(
    input: PathBuf,
    format: String,
    engine: String,
    languages: String,
    confidence: f32,
    classify: bool,
    analyze: bool,
    fast: bool,
) -> Result<()> {
    // Load image
    let image = ImageReader::open(&input)?
        .decode()
        .map_err(|e| anyhow::anyhow!("Failed to decode image {}: {}", input.display(), e))?;
    
    println!("Loaded image: {}x{} pixels", image.width(), image.height());
    
    // Configure OCR
    let languages: Vec<String> = languages.split(',').map(|s| s.trim().to_string()).collect();
    let mut config = OCRConfig {
        engine,
        languages,
        min_confidence: confidence,
        enable_text_classification: classify,
        enable_structure_analysis: analyze,
        ..Default::default()
    };
    
    // Apply fast mode optimizations
    if fast {
        config.preprocessing.enabled = false; // Skip preprocessing for speed
        config.min_confidence = 0.3; // Lower confidence threshold
        println!("Fast mode enabled: preprocessing disabled, lower confidence threshold");
    }
    
    // Create processor and extract text
    let processor = OCRProcessor::new(config)?;
    let result = processor.process_image(&image).await?;
    
    // Output results
    match format.as_str() {
        "json" => {
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        "text" => {
            for block in &result.text_blocks {
                println!("{}", block.text);
            }
        }
        "structured" => {
            println!("=== Text Blocks ===");
            for block in &result.text_blocks {
                println!("[{:?}] {} (confidence: {:.2})", 
                    block.semantic_type, block.text, block.confidence);
            }
            
            if !result.structured_content.code_blocks.is_empty() {
                println!("\n=== Code Blocks ===");
                for code_block in &result.structured_content.code_blocks {
                    println!("Language: {:?}", code_block.language);
                    println!("Content:\n{}", code_block.content);
                    println!("---");
                }
            }
            
            if !result.structured_content.ui_elements.is_empty() {
                println!("\n=== UI Elements ===");
                for ui_element in &result.structured_content.ui_elements {
                    println!("[{:?}] {} (interactive: {})", 
                        ui_element.element_type, ui_element.text, ui_element.is_interactive);
                }
            }
        }
        _ => {
            anyhow::bail!("Unsupported format: {}", format);
        }
    }
    
    Ok(())
}

async fn process_from_stdin(_format: String, _config: Option<PathBuf>) -> Result<()> {
    // Placeholder for stdin processing
    anyhow::bail!("Stdin processing not yet implemented");
}

async fn test_ocr(input: Option<PathBuf>) -> Result<()> {
    println!("Testing OCR capabilities...");
    
    let test_image = if let Some(path) = input {
        ImageReader::open(path)?.decode()?
    } else {
        // Create a simple test image with text
        use image::{ImageBuffer, Rgb};
        let img = ImageBuffer::from_fn(200, 100, |x, y| {
            if x > 10 && x < 190 && y > 40 && y < 60 {
                Rgb([0, 0, 0]) // Black text area
            } else {
                Rgb([255, 255, 255]) // White background
            }
        });
        image::DynamicImage::ImageRgb8(img)
    };
    
    let config = OCRConfig::default();
    let processor = OCRProcessor::new(config)?;
    
    let start_time = std::time::Instant::now();
    let result = processor.process_image(&test_image).await?;
    let processing_time = start_time.elapsed();
    
    println!("OCR Processing completed in {:?}", processing_time);
    println!("Detected {} text blocks", result.text_blocks.len());
    println!("Overall confidence: {:.2}", result.overall_confidence);
    println!("Primary language: {}", result.detected_language);
    
    if !result.text_blocks.is_empty() {
        println!("\nExtracted text:");
        for (i, block) in result.text_blocks.iter().enumerate() {
            println!("  {}: {} (confidence: {:.2})", i + 1, block.text, block.confidence);
        }
    }
    
    Ok(())
}