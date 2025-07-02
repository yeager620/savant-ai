use anyhow::Result;
use chrono::{DateTime, Utc, TimeZone};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use tokio::sync::RwLock;
use tracing::{debug, error, info};

use savant_db::{TranscriptDatabase, visual_data::VisualDataManager};

#[derive(Debug, Clone)]
pub struct VisualMCPServer {
    db: TranscriptDatabase,
    visual_db: VisualDataManager,
    capabilities: ServerCapabilities,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerCapabilities {
    pub supports_visual_queries: bool,
    pub supports_temporal_analysis: bool,
    pub supports_activity_tracking: bool,
    pub supports_assistance_history: bool,
    pub supports_real_time_context: bool,
}

impl Default for ServerCapabilities {
    fn default() -> Self {
        Self {
            supports_visual_queries: true,
            supports_temporal_analysis: true,
            supports_activity_tracking: true,
            supports_assistance_history: true,
            supports_real_time_context: true,
        }
    }
}

impl VisualMCPServer {
    pub async fn new(db_path: Option<std::path::PathBuf>) -> Result<Self> {
        let db = TranscriptDatabase::new(db_path).await?;
        let visual_db = VisualDataManager::new(db.pool.clone());
        
        Ok(Self {
            db,
            visual_db,
            capabilities: ServerCapabilities::default(),
        })
    }

    pub async fn handle_tool_call(&self, tool_name: &str, arguments: Value) -> Result<Value> {
        match tool_name {
            // Visual content queries
            "query_screen_content" => self.query_screen_content(arguments).await,
            "search_visual_text" => self.search_visual_text(arguments).await,
            "find_on_screen" => self.find_on_screen(arguments).await,
            
            // Temporal analysis
            "what_was_i_doing" => self.what_was_i_doing(arguments).await,
            "activity_timeline" => self.activity_timeline(arguments).await,
            "time_spent_analysis" => self.time_spent_analysis(arguments).await,
            
            // Application and context queries
            "application_usage" => self.application_usage(arguments).await,
            "context_switches" => self.context_switches(arguments).await,
            "productivity_analysis" => self.productivity_analysis(arguments).await,
            
            // Task and assistance queries
            "detected_tasks" => self.detected_tasks(arguments).await,
            "assistance_history" => self.assistance_history(arguments).await,
            "unanswered_questions" => self.unanswered_questions(arguments).await,
            
            // Real-time context
            "current_screen_analysis" => self.current_screen_analysis(arguments).await,
            "recent_activity" => self.recent_activity(arguments).await,
            "active_opportunities" => self.active_opportunities(arguments).await,
            
            // Code and development queries
            "code_analysis" => self.code_analysis(arguments).await,
            "error_patterns" => self.error_patterns(arguments).await,
            "development_insights" => self.development_insights(arguments).await,
            
            // Advanced analytics
            "attention_patterns" => self.attention_patterns(arguments).await,
            "focus_analysis" => self.focus_analysis(arguments).await,
            "distraction_tracking" => self.distraction_tracking(arguments).await,
            
            _ => Err(anyhow::anyhow!("Unknown tool: {}", tool_name)),
        }
    }

    pub async fn list_tools(&self) -> Value {
        json!([
            {
                "name": "query_screen_content",
                "description": "Query what was visible on screen during a specific time period",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "start_time": {"type": "string", "description": "Start time (ISO 8601)"},
                        "end_time": {"type": "string", "description": "End time (ISO 8601)"},
                        "application": {"type": "string", "description": "Filter by application name"},
                        "text_contains": {"type": "string", "description": "Filter by text content"},
                        "region_type": {"type": "string", "description": "Screen region: menubar, sidebar, main, statusbar"}
                    }
                }
            },
            {
                "name": "search_visual_text",
                "description": "Search for specific text that appeared on screen with context",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Text to search for"},
                        "semantic_type": {"type": "string", "description": "Type: ui, code, document, chat, error"},
                        "time_range": {"type": "string", "description": "Time range: today, yesterday, this_week, last_7_days"},
                        "min_confidence": {"type": "number", "description": "Minimum OCR confidence (0.0-1.0)"}
                    },
                    "required": ["query"]
                }
            },
            {
                "name": "what_was_i_doing",
                "description": "Analyze what you were doing during a specific time period",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "when": {"type": "string", "description": "Time description: '2 hours ago', 'yesterday at 3pm', 'this morning'"},
                        "duration": {"type": "string", "description": "Duration: '30 minutes', '1 hour', '2 hours'"},
                        "detail_level": {"type": "string", "description": "summary, detailed, comprehensive"}
                    },
                    "required": ["when"]
                }
            },
            {
                "name": "activity_timeline",
                "description": "Get a timeline of your activities for analysis and review",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "date": {"type": "string", "description": "Date (YYYY-MM-DD) or 'today', 'yesterday'"},
                        "group_by": {"type": "string", "description": "Group by: application, activity_type, hour"},
                        "include_tasks": {"type": "boolean", "description": "Include detected tasks"},
                        "include_questions": {"type": "boolean", "description": "Include detected questions"}
                    }
                }
            },
            {
                "name": "time_spent_analysis",
                "description": "Analyze how time was spent across applications and activities",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "period": {"type": "string", "description": "today, yesterday, this_week, last_7_days, this_month"},
                        "group_by": {"type": "string", "description": "application, activity_type, productivity_level"},
                        "min_duration_minutes": {"type": "number", "description": "Minimum duration to include"}
                    }
                }
            },
            {
                "name": "application_usage",
                "description": "Get detailed application usage statistics and patterns",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "period": {"type": "string", "description": "Time period to analyze"},
                        "sort_by": {"type": "string", "description": "time_spent, productivity, task_count, question_count"},
                        "include_productivity": {"type": "boolean", "description": "Include productivity metrics"}
                    }
                }
            },
            {
                "name": "detected_tasks",
                "description": "Query tasks that were automatically detected from screen content",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "task_type": {"type": "string", "description": "debugging, coding, research, writing, etc."},
                        "status": {"type": "string", "description": "detected, in_progress, completed, abandoned"},
                        "period": {"type": "string", "description": "Time period to search"},
                        "min_confidence": {"type": "number", "description": "Minimum detection confidence"}
                    }
                }
            },
            {
                "name": "assistance_history",
                "description": "Review history of assistance opportunities and responses",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "opportunity_type": {"type": "string", "description": "Type of assistance offered"},
                        "period": {"type": "string", "description": "Time period to analyze"},
                        "status": {"type": "string", "description": "active, presented, accepted, dismissed, expired"}
                    }
                }
            },
            {
                "name": "current_screen_analysis",
                "description": "Get real-time analysis of current screen content and context",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "include_text": {"type": "boolean", "description": "Include extracted text"},
                        "include_tasks": {"type": "boolean", "description": "Include detected tasks"},
                        "include_opportunities": {"type": "boolean", "description": "Include assistance opportunities"}
                    }
                }
            },
            {
                "name": "code_analysis",
                "description": "Analyze code-related activities and detected programming patterns",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "language": {"type": "string", "description": "Programming language filter"},
                        "period": {"type": "string", "description": "Time period to analyze"},
                        "complexity_level": {"type": "string", "description": "trivial, simple, moderate, complex, expert"},
                        "include_errors": {"type": "boolean", "description": "Include error analysis"}
                    }
                }
            },
            {
                "name": "focus_analysis",
                "description": "Analyze focus patterns and attention distribution",
                "inputSchema": {
                    "type": "object",
                    "properties": {
                        "period": {"type": "string", "description": "Time period to analyze"},
                        "metric": {"type": "string", "description": "focus_duration, task_switching, deep_work_blocks"},
                        "threshold_minutes": {"type": "number", "description": "Minimum duration for focus blocks"}
                    }
                }
            }
        ])
    }

    // Tool implementations

    async fn query_screen_content(&self, args: Value) -> Result<Value> {
        let start_time = args.get("start_time")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));
        
        let end_time = args.get("end_time")
            .and_then(|v| v.as_str())
            .and_then(|s| DateTime::parse_from_rfc3339(s).ok())
            .map(|dt| dt.with_timezone(&Utc));

        let application = args.get("application").and_then(|v| v.as_str()).map(String::from);
        let text_contains = args.get("text_contains").and_then(|v| v.as_str()).map(String::from);

        let query = savant_db::visual_data::VideoQuery {
            start_time,
            end_time,
            active_application: application,
            text_contains,
            limit: Some(50),
            ..Default::default()
        };

        let frames = self.visual_db.query_frames(&query).await?;
        
        Ok(json!({
            "frames_found": frames.len(),
            "time_range": {
                "start": start_time.map(|t| t.to_rfc3339()),
                "end": end_time.map(|t| t.to_rfc3339())
            },
            "frames": frames
        }))
    }

    async fn search_visual_text(&self, args: Value) -> Result<Value> {
        let query = args.get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required 'query' parameter"))?;

        let limit = args.get("limit")
            .and_then(|v| v.as_i64())
            .unwrap_or(20) as i64;

        let results = self.visual_db.search_text(query, Some(limit)).await?;

        Ok(json!({
            "query": query,
            "results_found": results.len(),
            "results": results
        }))
    }

    async fn what_was_i_doing(&self, args: Value) -> Result<Value> {
        let when = args.get("when")
            .and_then(|v| v.as_str())
            .ok_or_else(|| anyhow::anyhow!("Missing required 'when' parameter"))?;

        let duration = args.get("duration")
            .and_then(|v| v.as_str())
            .unwrap_or("1 hour");

        // Parse time expressions (simplified implementation)
        let (start_time, end_time) = self.parse_time_expression(when, duration)?;

        let query = savant_db::visual_data::VideoQuery {
            start_time: Some(start_time),
            end_time: Some(end_time),
            limit: Some(100),
            ..Default::default()
        };

        let frames = self.visual_db.query_frames(&query).await?;
        
        // Analyze activity patterns
        let mut app_time: HashMap<String, u64> = HashMap::new();
        let mut activities = Vec::new();
        
        for frame in &frames {
            if let Some(app) = frame["active_application"].as_str() {
                *app_time.entry(app.to_string()).or_insert(0) += 500; // 500ms per frame
            }
            
            if frame["ocr_blocks"].as_i64().unwrap_or(0) > 0 {
                activities.push(format!("Text activity in {}", 
                    frame["active_application"].as_str().unwrap_or("Unknown")));
            }
        }

        let mut sorted_apps: Vec<_> = app_time.into_iter().collect();
        sorted_apps.sort_by(|a, b| b.1.cmp(&a.1));

        Ok(json!({
            "time_period": {
                "start": start_time.to_rfc3339(),
                "end": end_time.to_rfc3339(),
                "requested": when,
                "duration": duration
            },
            "summary": {
                "total_frames": frames.len(),
                "primary_application": sorted_apps.first().map(|(app, _)| app),
                "total_applications": sorted_apps.len(),
                "activity_level": if frames.len() > 50 { "High" } else { "Moderate" }
            },
            "application_breakdown": sorted_apps.into_iter()
                .take(10)
                .map(|(app, ms)| json!({
                    "application": app,
                    "duration_minutes": ms / 60000,
                    "percentage": (ms as f64 / (end_time.timestamp() - start_time.timestamp()) as f64 * 1000.0 * 100.0) as u32
                }))
                .collect::<Vec<_>>(),
            "activities": activities.into_iter().take(20).collect::<Vec<_>>()
        }))
    }

    async fn activity_timeline(&self, args: Value) -> Result<Value> {
        let date = args.get("date")
            .and_then(|v| v.as_str())
            .unwrap_or("today");

        let (start_time, end_time) = self.parse_date_range(date)?;

        let query = savant_db::visual_data::VideoQuery {
            start_time: Some(start_time),
            end_time: Some(end_time),
            limit: Some(500),
            ..Default::default()
        };

        let frames = self.visual_db.query_frames(&query).await?;
        
        // Group frames by hour
        let mut hourly_activity = HashMap::new();
        
        for frame in &frames {
            if let Some(timestamp_str) = frame["timestamp"].as_str() {
                if let Ok(timestamp) = DateTime::parse_from_rfc3339(timestamp_str) {
                    let hour = timestamp.hour();
                    let entry = hourly_activity.entry(hour).or_insert_with(|| json!({
                        "hour": hour,
                        "applications": HashMap::<String, u32>::new(),
                        "frame_count": 0,
                        "task_count": 0,
                        "question_count": 0
                    }));
                    
                    entry["frame_count"] = json!(entry["frame_count"].as_u64().unwrap_or(0) + 1);
                    
                    if let Some(app) = frame["active_application"].as_str() {
                        let apps = entry["applications"].as_object_mut().unwrap();
                        let count = apps.get(app).and_then(|v| v.as_u64()).unwrap_or(0);
                        apps.insert(app.to_string(), json!(count + 1));
                    }
                }
            }
        }

        let mut timeline: Vec<_> = hourly_activity.into_values().collect();
        timeline.sort_by(|a, b| a["hour"].as_u64().cmp(&b["hour"].as_u64()));

        Ok(json!({
            "date": date,
            "time_range": {
                "start": start_time.to_rfc3339(),
                "end": end_time.to_rfc3339()
            },
            "timeline": timeline,
            "summary": {
                "total_frames": frames.len(),
                "active_hours": timeline.len(),
                "peak_activity_hour": timeline.iter()
                    .max_by_key(|entry| entry["frame_count"].as_u64().unwrap_or(0))
                    .and_then(|entry| entry["hour"].as_u64())
            }
        }))
    }

    async fn application_usage(&self, args: Value) -> Result<Value> {
        let period = args.get("period")
            .and_then(|v| v.as_str())
            .unwrap_or("today");

        let usage_stats = self.visual_db.get_application_usage(Some(20)).await?;

        Ok(json!({
            "period": period,
            "applications": usage_stats.into_iter().map(|usage| json!({
                "name": usage.application,
                "frame_count": usage.frame_count,
                "first_seen": usage.first_seen.to_rfc3339(),
                "last_seen": usage.last_seen.to_rfc3339(),
                "avg_productivity": usage.avg_productivity,
                "estimated_minutes": usage.frame_count * 500 / 60000 // Rough estimate
            })).collect::<Vec<_>>()
        }))
    }

    async fn detected_tasks(&self, args: Value) -> Result<Value> {
        let task_type = args.get("task_type").and_then(|v| v.as_str());
        let period = args.get("period").and_then(|v| v.as_str()).unwrap_or("today");

        // This would need to be implemented in the visual_db
        // For now, return a placeholder response
        Ok(json!({
            "task_type_filter": task_type,
            "period": period,
            "tasks": [],
            "summary": {
                "total_tasks": 0,
                "by_type": {},
                "completion_rate": 0.0
            }
        }))
    }

    async fn current_screen_analysis(&self, args: Value) -> Result<Value> {
        let include_text = args.get("include_text").and_then(|v| v.as_bool()).unwrap_or(true);
        let include_tasks = args.get("include_tasks").and_then(|v| v.as_bool()).unwrap_or(true);

        // Get most recent frame
        let now = Utc::now();
        let five_minutes_ago = now - chrono::Duration::minutes(5);

        let query = savant_db::visual_data::VideoQuery {
            start_time: Some(five_minutes_ago),
            end_time: Some(now),
            limit: Some(1),
            ..Default::default()
        };

        let frames = self.visual_db.query_frames(&query).await?;
        let current_frame = frames.first();

        let mut analysis = json!({
            "timestamp": now.to_rfc3339(),
            "frame_available": current_frame.is_some()
        });

        if let Some(frame) = current_frame {
            analysis["current_context"] = json!({
                "application": frame["active_application"],
                "resolution": {
                    "width": frame["resolution_width"],
                    "height": frame["resolution_height"]
                },
                "change_detected": frame["change_detected"],
                "primary_app_type": frame["primary_app_type"]
            });

            if include_text {
                analysis["text_content"] = json!({
                    "ocr_blocks": frame["ocr_blocks"],
                    "has_text": frame["ocr_blocks"].as_i64().unwrap_or(0) > 0
                });
            }

            if include_tasks {
                analysis["assistance"] = json!({
                    "opportunities": frame["opportunities"],
                    "has_opportunities": frame["opportunities"].as_i64().unwrap_or(0) > 0
                });
            }
        }

        Ok(analysis)
    }

    async fn code_analysis(&self, args: Value) -> Result<Value> {
        let language = args.get("language").and_then(|v| v.as_str());
        let period = args.get("period").and_then(|v| v.as_str()).unwrap_or("today");

        let code_stats = self.visual_db.get_code_analysis().await?;

        let filtered_stats: Vec<_> = code_stats.into_iter()
            .filter(|stat| language.map_or(true, |lang| stat.programming_language.contains(lang)))
            .collect();

        Ok(json!({
            "language_filter": language,
            "period": period,
            "languages": filtered_stats.into_iter().map(|stat| json!({
                "language": stat.programming_language,
                "snippet_count": stat.snippet_count,
                "avg_complexity": stat.avg_complexity,
                "unique_frames": stat.unique_frames,
                "first_detected": stat.first_detected.to_rfc3339(),
                "last_detected": stat.last_detected.to_rfc3339()
            })).collect::<Vec<_>>(),
            "summary": {
                "total_languages": filtered_stats.len(),
                "most_used_language": filtered_stats.first().map(|s| &s.programming_language)
            }
        }))
    }

    async fn assistance_history(&self, args: Value) -> Result<Value> {
        let period = args.get("period").and_then(|v| v.as_str()).unwrap_or("today");
        let opportunity_type = args.get("opportunity_type").and_then(|v| v.as_str());

        let opportunities = self.visual_db.get_opportunities(opportunity_type, Some(50)).await?;

        Ok(json!({
            "period": period,
            "opportunity_type_filter": opportunity_type,
            "opportunities": opportunities,
            "summary": {
                "total_opportunities": opportunities.len(),
                "by_urgency": {
                    "high": opportunities.iter().filter(|o| o["urgency"].as_str() == Some("High")).count(),
                    "medium": opportunities.iter().filter(|o| o["urgency"].as_str() == Some("Medium")).count(),
                    "low": opportunities.iter().filter(|o| o["urgency"].as_str() == Some("Low")).count()
                }
            }
        }))
    }

    async fn recent_activity(&self, args: Value) -> Result<Value> {
        let minutes = args.get("minutes").and_then(|v| v.as_i64()).unwrap_or(30);
        
        let now = Utc::now();
        let start_time = now - chrono::Duration::minutes(minutes);

        let query = savant_db::visual_data::VideoQuery {
            start_time: Some(start_time),
            end_time: Some(now),
            limit: Some(100),
            ..Default::default()
        };

        let frames = self.visual_db.query_frames(&query).await?;

        Ok(json!({
            "time_window_minutes": minutes,
            "frames_analyzed": frames.len(),
            "period": {
                "start": start_time.to_rfc3339(),
                "end": now.to_rfc3339()
            },
            "recent_frames": frames.into_iter().take(10).collect::<Vec<_>>()
        }))
    }

    // Placeholder implementations for additional methods
    async fn time_spent_analysis(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Time spent analysis not yet implemented"}))
    }

    async fn context_switches(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Context switches analysis not yet implemented"}))
    }

    async fn productivity_analysis(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Productivity analysis not yet implemented"}))
    }

    async fn unanswered_questions(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Unanswered questions not yet implemented"}))
    }

    async fn active_opportunities(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Active opportunities not yet implemented"}))
    }

    async fn error_patterns(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Error patterns not yet implemented"}))
    }

    async fn development_insights(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Development insights not yet implemented"}))
    }

    async fn attention_patterns(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Attention patterns not yet implemented"}))
    }

    async fn focus_analysis(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Focus analysis not yet implemented"}))
    }

    async fn distraction_tracking(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Distraction tracking not yet implemented"}))
    }

    async fn find_on_screen(&self, _args: Value) -> Result<Value> {
        Ok(json!({"message": "Find on screen not yet implemented"}))
    }

    // Helper methods
    fn parse_time_expression(&self, when: &str, duration: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
        let now = Utc::now();
        
        // Simple parsing - could be much more sophisticated
        let start_time = match when {
            "now" => now,
            "1 hour ago" => now - chrono::Duration::hours(1),
            "2 hours ago" => now - chrono::Duration::hours(2),
            "this morning" => {
                let today = now.date_naive();
                Utc.from_utc_datetime(&today.and_hms_opt(9, 0, 0).unwrap())
            },
            "yesterday" => now - chrono::Duration::days(1),
            _ => now - chrono::Duration::hours(1), // Default
        };

        let duration_parsed = match duration {
            "30 minutes" => chrono::Duration::minutes(30),
            "1 hour" => chrono::Duration::hours(1),
            "2 hours" => chrono::Duration::hours(2),
            _ => chrono::Duration::hours(1), // Default
        };

        let end_time = start_time + duration_parsed;

        Ok((start_time, end_time))
    }

    fn parse_date_range(&self, date: &str) -> Result<(DateTime<Utc>, DateTime<Utc>)> {
        let now = Utc::now();
        
        let (start, end) = match date {
            "today" => {
                let today = now.date_naive();
                let start = Utc.from_utc_datetime(&today.and_hms_opt(0, 0, 0).unwrap());
                let end = Utc.from_utc_datetime(&today.and_hms_opt(23, 59, 59).unwrap());
                (start, end)
            },
            "yesterday" => {
                let yesterday = (now - chrono::Duration::days(1)).date_naive();
                let start = Utc.from_utc_datetime(&yesterday.and_hms_opt(0, 0, 0).unwrap());
                let end = Utc.from_utc_datetime(&yesterday.and_hms_opt(23, 59, 59).unwrap());
                (start, end)
            },
            _ => {
                // Try to parse as YYYY-MM-DD
                if let Ok(date_naive) = chrono::NaiveDate::parse_from_str(date, "%Y-%m-%d") {
                    let start = Utc.from_utc_datetime(&date_naive.and_hms_opt(0, 0, 0).unwrap());
                    let end = Utc.from_utc_datetime(&date_naive.and_hms_opt(23, 59, 59).unwrap());
                    (start, end)
                } else {
                    // Default to today
                    let today = now.date_naive();
                    let start = Utc.from_utc_datetime(&today.and_hms_opt(0, 0, 0).unwrap());
                    let end = Utc.from_utc_datetime(&today.and_hms_opt(23, 59, 59).unwrap());
                    (start, end)
                }
            }
        };

        Ok((start, end))
    }
}