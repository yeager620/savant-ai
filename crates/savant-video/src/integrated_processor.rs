use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};

use crate::{
    VideoFrame,
    real_time_analyzer::RealTimeAnalyzer,
    coding_problem_detector::{CodingProblemDetector, DetectedCodingProblem, DetectionConfig},
    solution_generator::{SolutionGenerator, GeneratedSolution, SolutionConfig},
    change_detector::ChangeDetector,
};
use savant_ocr::ComprehensiveOCRProcessor;
use savant_vision::VisionAnalyzer;
use crate::llm_provider::LLMProvider;
use sqlx::SqlitePool;

#[derive(Debug)]
pub struct IntegratedProcessor {
    pub config: ProcessorConfig,
    pub ocr_processor: ComprehensiveOCRProcessor,
    pub vision_analyzer: VisionAnalyzer,
    pub change_detector: ChangeDetector,
    pub real_time_analyzer: RealTimeAnalyzer,
    pub problem_detector: CodingProblemDetector,
    pub solution_generator: SolutionGenerator,
    pub db_pool: SqlitePool,
    pub event_tx: mpsc::UnboundedSender<ProcessingEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    pub enable_ocr: bool,
    pub enable_vision: bool,
    pub enable_real_time_analysis: bool,
    pub enable_problem_detection: bool,
    pub enable_auto_solutions: bool,
    pub min_change_threshold: f32,
    pub processing_timeout_ms: u64,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            enable_ocr: true,
            enable_vision: true,
            enable_real_time_analysis: true,
            enable_problem_detection: true,
            enable_auto_solutions: true,
            min_change_threshold: 0.05,
            processing_timeout_ms: 5000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingEvent {
    FrameProcessed {
        frame_id: String,
        timestamp: DateTime<Utc>,
        has_changes: bool,
    },
    TextExtracted {
        frame_id: String,
        word_count: usize,
        paragraphs: usize,
    },
    TaskDetected {
        task: crate::real_time_analyzer::DetectedTask,
    },
    QuestionDetected {
        question: crate::real_time_analyzer::DetectedQuestion,
    },
    CodingProblemDetected {
        problem: DetectedCodingProblem,
    },
    SolutionGenerated {
        solution: GeneratedSolution,
        problem_id: String,
    },
    ProcessingError {
        frame_id: String,
        error: String,
    },
}

impl IntegratedProcessor {
    pub async fn new(
        config: ProcessorConfig,
        llm_provider: LLMProvider,
        db_pool: sqlx::SqlitePool,
    ) -> Result<(Self, mpsc::UnboundedReceiver<ProcessingEvent>)> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let processor = Self {
            config,
            ocr_processor: ComprehensiveOCRProcessor::new(Default::default()),
            vision_analyzer: VisionAnalyzer::new(Default::default())?,
            change_detector: ChangeDetector::new(Default::default()),
            real_time_analyzer: RealTimeAnalyzer::new(Default::default()),
            problem_detector: CodingProblemDetector::new(DetectionConfig::default()),
            solution_generator: SolutionGenerator::new(
                SolutionConfig::default(),
                llm_provider,
            ),
            db_pool,
            event_tx,
        };

        Ok((processor, event_rx))
    }

    pub async fn process_frame(&mut self, frame: &VideoFrame) -> Result<ProcessingResult> {
        let start_time = std::time::Instant::now();
        info!("Processing frame: {}", frame.id);

        // Load image
        let image = image::open(&frame.file_path)?;

        // Check for changes  
        let image_bytes = {
            let mut bytes = Vec::new();
            image.write_to(&mut std::io::Cursor::new(&mut bytes), image::ImageFormat::Png)?;
            bytes
        };
        let change_result = self.change_detector.detect_changes(frame.clone(), image_bytes, None).await?;

        if !change_result.significant_change && change_result.change_score < self.config.min_change_threshold {
            debug!("No significant changes detected, skipping processing");
            self.event_tx.send(ProcessingEvent::FrameProcessed {
                frame_id: frame.id.clone(),
                timestamp: frame.timestamp,
                has_changes: false,
            })?;

            return Ok(ProcessingResult {
                frame_id: frame.id.clone(),
                processing_time_ms: start_time.elapsed().as_millis() as u64,
                changes_detected: false,
                text_extracted: None,
                vision_analysis: None,
                detected_tasks: vec![],
                detected_problems: vec![],
                generated_solutions: vec![],
            });
        }

        let mut result = ProcessingResult {
            frame_id: frame.id.clone(),
            processing_time_ms: 0,
            changes_detected: true,
            text_extracted: None,
            vision_analysis: None,
            detected_tasks: vec![],
            detected_problems: vec![],
            generated_solutions: vec![],
        };

        // Run OCR if enabled
        let ocr_result = if self.config.enable_ocr {
            match self.ocr_processor.process_image(&image).await {
                Ok(ocr) => {
                    self.event_tx.send(ProcessingEvent::TextExtracted {
                        frame_id: frame.id.clone(),
                        word_count: ocr.words.len(),
                        paragraphs: ocr.paragraphs.len(),
                    })?;

                    // Store in database
                    self.store_text_extractions(&frame.id, &ocr).await?;

                    result.text_extracted = Some(TextExtractionSummary {
                        total_words: ocr.words.len(),
                        total_paragraphs: ocr.paragraphs.len(),
                        screen_regions: ocr.screen_regions.len(),
                    });

                    Some(ocr)
                }
                Err(e) => {
                    warn!("OCR processing failed: {}", e);
                    self.event_tx.send(ProcessingEvent::ProcessingError {
                        frame_id: frame.id.clone(),
                        error: format!("OCR failed: {}", e),
                    })?;
                    None
                }
            }
        } else {
            None
        };

        // Run vision analysis if enabled
        let vision_result = if self.config.enable_vision {
            match self.vision_analyzer.analyze_screen(&image).await {
                Ok(analysis) => {
                    result.vision_analysis = Some(VisionAnalysisSummary {
                        detected_apps: analysis.app_context.detected_applications.len(),
                        activity_type: Some(format!("{:?}", analysis.activity_classification.primary_activity)),
                        confidence: analysis.activity_classification.confidence,
                    });

                    Some(analysis)
                }
                Err(e) => {
                    warn!("Vision analysis failed: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // Run real-time analysis if enabled
        if self.config.enable_real_time_analysis {
            if let (Some(ocr), Some(vision)) = (&ocr_result, &vision_result) {
                match self.real_time_analyzer.analyze_frame(ocr.clone(), Some(vision.clone())).await {
                    Ok(task_result) => {
                        // Send events for detected tasks
                        for task in &task_result.detected_tasks {
                            self.event_tx.send(ProcessingEvent::TaskDetected {
                                task: task.clone(),
                            })?;
                        }

                        // Send events for detected questions
                        for question in &task_result.detected_questions {
                            self.event_tx.send(ProcessingEvent::QuestionDetected {
                                question: question.clone(),
                            })?;
                        }

                        result.detected_tasks = task_result.detected_tasks.clone();

                        // Store in database
                        self.store_detected_tasks(&frame.id, &task_result).await?;
                    }
                    Err(e) => {
                        warn!("Real-time analysis failed: {}", e);
                    }
                }
            }
        }

        // Run coding problem detection if enabled
        if self.config.enable_problem_detection {
            if let (Some(ocr), Some(vision)) = (&ocr_result, &vision_result) {
                match self.problem_detector.detect_problems(ocr, vision).await {
                    Ok(problems) => {
                        for problem in &problems {
                            info!("Detected coding problem: {} - {}", problem.problem_type.to_string(), problem.title);

                            self.event_tx.send(ProcessingEvent::CodingProblemDetected {
                                problem: problem.clone(),
                            })?;

                            // Generate solution if auto-solutions enabled
                            if self.config.enable_auto_solutions {
                                match self.solution_generator.generate_solution(problem).await {
                                    Ok(solution) => {
                                        info!("Generated solution for problem: {}", problem.id);

                                        self.event_tx.send(ProcessingEvent::SolutionGenerated {
                                            solution: solution.clone(),
                                            problem_id: problem.id.clone(),
                                        })?;

                                        result.generated_solutions.push(solution);
                                    }
                                    Err(e) => {
                                        error!("Failed to generate solution: {}", e);
                                    }
                                }
                            }
                        }

                        result.detected_problems = problems;
                    }
                    Err(e) => {
                        warn!("Problem detection failed: {}", e);
                    }
                }
            }
        }

        // Store frame metadata
        self.store_frame_metadata(frame, &result).await?;

        result.processing_time_ms = start_time.elapsed().as_millis() as u64;

        self.event_tx.send(ProcessingEvent::FrameProcessed {
            frame_id: frame.id.clone(),
            timestamp: frame.timestamp,
            has_changes: true,
        })?;

        Ok(result)
    }

    async fn store_text_extractions(
        &self,
        frame_id: &str,
        ocr_result: &savant_ocr::ComprehensiveOCRResult,
    ) -> Result<()> {
        // Store text extractions in the high-frequency database
        for (_word_idx, word) in ocr_result.words.iter().enumerate() {
            sqlx::query(
                r#"
                INSERT INTO hf_text_extractions (
                    frame_id, word_text, confidence, 
                    bbox_x, bbox_y, bbox_width, bbox_height,
                    font_size_estimate, text_type, line_id, paragraph_id
                ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(frame_id)
            .bind(&word.text)
            .bind(word.confidence)
            .bind(word.bounding_box.x)
            .bind(word.bounding_box.y)
            .bind(word.bounding_box.width)
            .bind(word.bounding_box.height)
            .bind(word.font_size_estimate)
            .bind(word.text_type.as_ref().map(|t| format!("{:?}", t)))
            .bind(word.line_id as i32)
            .bind(word.paragraph_id as i32)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    async fn store_detected_tasks(
        &self,
        frame_id: &str,
        task_result: &crate::real_time_analyzer::TaskDetectionResult,
    ) -> Result<()> {
        // Store detected tasks
        for task in &task_result.detected_tasks {
            sqlx::query(
                r#"
                INSERT INTO hf_detected_tasks (
                    frame_id, task_type, confidence, description,
                    evidence_text, bounding_regions, assistance_suggestions
                ) VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(frame_id)
            .bind(format!("{:?}", task.task_type))
            .bind(task.confidence)
            .bind(&task.description)
            .bind(serde_json::to_string(&task.context)?)
            .bind(task.bounding_box.as_ref().map(|b| serde_json::to_string(b).unwrap_or_default()))
            .bind(serde_json::to_string(&task.suggested_assistance)?)
            .execute(&self.db_pool)
            .await?;
        }

        Ok(())
    }

    async fn store_frame_metadata(
        &self,
        frame: &VideoFrame,
        result: &ProcessingResult,
    ) -> Result<()> {
        // Store high-frequency frame data
        sqlx::query(
            r#"
            INSERT INTO hf_video_frames (
                timestamp_ms, session_id, frame_hash, change_score,
                file_path, screen_resolution, active_app, processing_flags
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(frame.timestamp.timestamp_millis())
        .bind(&frame.metadata.session_id)
        .bind(&frame.image_hash)
        .bind(0.5) // placeholder change score
        .bind(frame.file_path.to_string_lossy())
        .bind(format!("{}x{}", frame.resolution.0, frame.resolution.1))
        .bind(&frame.metadata.active_application)
        .bind(1) // processing flags
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub frame_id: String,
    pub processing_time_ms: u64,
    pub changes_detected: bool,
    pub text_extracted: Option<TextExtractionSummary>,
    pub vision_analysis: Option<VisionAnalysisSummary>,
    pub detected_tasks: Vec<crate::real_time_analyzer::DetectedTask>,
    pub detected_problems: Vec<DetectedCodingProblem>,
    pub generated_solutions: Vec<GeneratedSolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextExtractionSummary {
    pub total_words: usize,
    pub total_paragraphs: usize,
    pub screen_regions: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisionAnalysisSummary {
    pub detected_apps: usize,
    pub activity_type: Option<String>,
    pub confidence: f32,
}
