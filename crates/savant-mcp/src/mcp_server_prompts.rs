//! MCP server prompt implementations for structured workflows
//! 
//! Provides templated prompts for common conversation analysis tasks

use anyhow::{anyhow, Result};
use sqlx::Row;
use serde_json::{json, Value};

use crate::mcp_server::MCPServer;

impl MCPServer {
    /// Handle list prompts request
    pub async fn handle_list_prompts(&self) -> Result<Value> {
        let prompts = vec![
            json!({
                "name": "analyze_conversation",
                "description": "Analyze a specific conversation for key insights, topics, and participant dynamics",
                "arguments": [
                    {
                        "name": "conversation_id",
                        "description": "ID of the conversation to analyze",
                        "required": true
                    },
                    {
                        "name": "analysis_type",
                        "description": "Type of analysis: topics, sentiment, dynamics, or summary",
                        "required": false
                    }
                ]
            }),
            json!({
                "name": "compare_speakers",
                "description": "Compare speaking patterns and interaction styles between two or more speakers",
                "arguments": [
                    {
                        "name": "speakers",
                        "description": "Comma-separated list of speaker names to compare",
                        "required": true
                    },
                    {
                        "name": "time_period",
                        "description": "Time period for comparison (e.g., 'last week', '2025-01-01 to 2025-01-31')",
                        "required": false
                    }
                ]
            }),
            json!({
                "name": "find_topic_evolution",
                "description": "Track how specific topics or themes have evolved over time in conversations",
                "arguments": [
                    {
                        "name": "topic",
                        "description": "Topic or keyword to track",
                        "required": true
                    },
                    {
                        "name": "time_range",
                        "description": "Time range for tracking (e.g., 'last 30 days', 'this month')",
                        "required": false
                    }
                ]
            }),
            json!({
                "name": "weekly_summary",
                "description": "Generate a comprehensive weekly summary of conversation activity and insights",
                "arguments": [
                    {
                        "name": "week_offset",
                        "description": "Number of weeks back from current (0 = this week, 1 = last week, etc.)",
                        "required": false
                    },
                    {
                        "name": "include_metrics",
                        "description": "Include detailed metrics and statistics",
                        "required": false
                    }
                ]
            }),
            json!({
                "name": "meeting_preparation",
                "description": "Prepare for upcoming meetings by analyzing past discussions with participants",
                "arguments": [
                    {
                        "name": "participants",
                        "description": "Expected meeting participants (comma-separated)",
                        "required": true
                    },
                    {
                        "name": "topics",
                        "description": "Expected meeting topics or agenda items",
                        "required": false
                    }
                ]
            }),
            json!({
                "name": "action_items_tracker",
                "description": "Extract and track action items and commitments from conversations",
                "arguments": [
                    {
                        "name": "speaker",
                        "description": "Specific speaker to track action items for (optional)",
                        "required": false
                    },
                    {
                        "name": "status",
                        "description": "Action item status: pending, completed, or overdue",
                        "required": false
                    }
                ]
            })
        ];
        
        Ok(json!({ "prompts": prompts }))
    }
    
    /// Handle get prompt request
    pub async fn handle_get_prompt(&self, params: Option<Value>) -> Result<Value> {
        let params = params.ok_or_else(|| anyhow!("Missing prompt parameters"))?;
        
        let prompt_name = params.get("name")
            .and_then(|n| n.as_str())
            .ok_or_else(|| anyhow!("Missing prompt name"))?;
            
        let default_args = json!({});
        let arguments = params.get("arguments")
            .unwrap_or(&default_args);
        
        match prompt_name {
            "analyze_conversation" => self.prompt_analyze_conversation(arguments).await,
            "compare_speakers" => self.prompt_compare_speakers(arguments).await,
            "find_topic_evolution" => self.prompt_find_topic_evolution(arguments).await,
            "weekly_summary" => self.prompt_weekly_summary(arguments).await,
            "meeting_preparation" => self.prompt_meeting_preparation(arguments).await,
            "action_items_tracker" => self.prompt_action_items_tracker(arguments).await,
            _ => Err(anyhow!("Unknown prompt: {}", prompt_name)),
        }
    }
    
    /// Analyze conversation prompt
    async fn prompt_analyze_conversation(&self, args: &Value) -> Result<Value> {
        let conversation_id = args.get("conversation_id")
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Missing conversation_id parameter"))?;
            
        let analysis_type = args.get("analysis_type")
            .and_then(|t| t.as_str())
            .unwrap_or("summary");
        
        // Get conversation data
        let conversation = self.database.get_conversation(conversation_id).await?
            .ok_or_else(|| anyhow!("Conversation not found"))?;
            
        let segments = self.database.get_conversation_segments(conversation_id).await?;
        
        // Build conversation transcript
        let transcript = segments
            .iter()
            .map(|segment| {
                let speaker = segment.get("speaker")
                    .and_then(|s| s.as_str())
                    .unwrap_or("Unknown");
                let text = segment.get("text")
                    .and_then(|t| t.as_str())
                    .unwrap_or("");
                let timestamp = segment.get("timestamp")
                    .and_then(|ts| ts.as_str())
                    .unwrap_or("");
                
                format!("[{}] {}: {}", timestamp, speaker, text)
            })
            .collect::<Vec<_>>()
            .join("\n");
        
        let prompt_template = match analysis_type {
            "topics" => {
                format!(r#"Analyze the following conversation transcript and identify the main topics discussed:

Conversation: {}
Transcript:
{}

Please provide:
1. Main topics and themes
2. Key discussion points for each topic
3. Topic progression and flow
4. Any unresolved topics or questions

Format your response as a structured analysis."#, 
                    conversation.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled"),
                    transcript)
            },
            "sentiment" => {
                format!(r#"Analyze the sentiment and emotional tone of this conversation:

Conversation: {}
Transcript:
{}

Please provide:
1. Overall sentiment of the conversation
2. Individual speaker sentiment patterns
3. Emotional shifts or turning points
4. Tone indicators (formal, casual, tense, collaborative, etc.)
5. Any notable emotional moments

Format your response as a sentiment analysis report."#,
                    conversation.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled"),
                    transcript)
            },
            "dynamics" => {
                format!(r#"Analyze the participant dynamics and interaction patterns in this conversation:

Conversation: {}
Transcript:
{}

Please provide:
1. Speaking time distribution among participants
2. Interaction patterns (who talks to whom, response patterns)
3. Leadership and influence indicators
4. Collaboration vs. conflict indicators
5. Communication styles of each participant

Format your response as a dynamics analysis report."#,
                    conversation.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled"),
                    transcript)
            },
            _ => { // summary
                format!(r#"Provide a comprehensive summary and analysis of this conversation:

Conversation: {}
Transcript:
{}

Please provide:
1. Executive summary (2-3 sentences)
2. Key points discussed
3. Decisions made or conclusions reached
4. Action items or next steps mentioned
5. Important quotes or insights
6. Overall assessment

Format your response as a structured conversation summary."#,
                    conversation.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled"),
                    transcript)
            }
        };
        
        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_template
                    }
                }
            ],
            "metadata": {
                "conversation_id": conversation_id,
                "analysis_type": analysis_type,
                "segment_count": segments.len(),
                "conversation_title": conversation.get("title")
            }
        }))
    }
    
    /// Compare speakers prompt
    async fn prompt_compare_speakers(&self, args: &Value) -> Result<Value> {
        let speakers_str = args.get("speakers")
            .and_then(|s| s.as_str())
            .ok_or_else(|| anyhow!("Missing speakers parameter"))?;
            
        let speakers: Vec<&str> = speakers_str.split(',').map(|s| s.trim()).collect();
        let time_period = args.get("time_period")
            .and_then(|t| t.as_str())
            .unwrap_or("all time");
        
        // Get analytics for each speaker
        let mut speaker_data = Vec::new();
        for speaker in &speakers {
            let analytics_query = r#"
                SELECT * FROM speaker_analytics WHERE speaker = ?
            "#;
            
            if let Ok(Some(analytics)) = sqlx::query(analytics_query)
                .bind(speaker)
                .fetch_optional(&self.database.pool)
                .await 
            {
                speaker_data.push(json!({
                    "speaker": speaker,
                    "conversation_count": analytics.get::<i64, _>("conversation_count"),
                    "total_segments": analytics.get::<i64, _>("total_segments"),
                    "total_duration": analytics.get::<f64, _>("total_duration"),
                    "avg_confidence": analytics.get::<f64, _>("avg_confidence"),
                    "active_days": analytics.get::<i64, _>("active_days")
                }));
            }
        }
        
        // Get interaction data between speakers
        let interaction_query = r#"
            SELECT speaker_a, speaker_b, interaction_count, total_duration
            FROM speaker_interaction_matrix
            WHERE (speaker_a IN ({}) AND speaker_b IN ({}))
            ORDER BY interaction_count DESC
        "#;
        
        let placeholders = speakers.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let query = interaction_query.replace("{}", &placeholders);
        
        let mut query_builder = sqlx::query(&query);
        for speaker in &speakers {
            query_builder = query_builder.bind(speaker);
        }
        for speaker in &speakers {
            query_builder = query_builder.bind(speaker);
        }
        
        let interactions = query_builder.fetch_all(&self.database.pool).await.unwrap_or_default();
        
        let interaction_data: Vec<Value> = interactions
            .into_iter()
            .map(|row| {
                json!({
                    "speaker_a": row.get::<String, _>("speaker_a"),
                    "speaker_b": row.get::<String, _>("speaker_b"),
                    "interaction_count": row.get::<i64, _>("interaction_count"),
                    "total_duration": row.get::<f64, _>("total_duration")
                })
            })
            .collect();
        
        let prompt_template = format!(r#"Compare and analyze the speaking patterns and characteristics of these speakers:

Time Period: {}

Speaker Analytics:
{}

Speaker Interactions:
{}

Please provide a comprehensive comparison including:
1. Speaking frequency and volume comparison
2. Communication style differences
3. Collaboration patterns between speakers
4. Individual strengths and characteristics
5. Interaction dynamics and relationships
6. Recommendations for improved collaboration

Format your response as a structured speaker comparison report."#,
            time_period,
            serde_json::to_string_pretty(&speaker_data)?,
            serde_json::to_string_pretty(&interaction_data)?
        );
        
        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_template
                    }
                }
            ],
            "metadata": {
                "speakers": speakers,
                "time_period": time_period,
                "speaker_count": speakers.len(),
                "interaction_count": interaction_data.len()
            }
        }))
    }
    
    /// Find topic evolution prompt
    async fn prompt_find_topic_evolution(&self, args: &Value) -> Result<Value> {
        let topic = args.get("topic")
            .and_then(|t| t.as_str())
            .ok_or_else(|| anyhow!("Missing topic parameter"))?;
            
        let time_range = args.get("time_range")
            .and_then(|t| t.as_str())
            .unwrap_or("last 30 days");
        
        // Search for topic mentions over time
        let search_query = r#"
            SELECT 
                s.text,
                s.speaker,
                s.timestamp,
                c.title as conversation_title,
                c.id as conversation_id,
                DATE(s.timestamp) as discussion_date
            FROM segments s
            JOIN conversations c ON s.conversation_id = c.id
            WHERE (s.text LIKE ? OR s.processed_text LIKE ?)
              AND s.timestamp > datetime('now', '-30 days')
            ORDER BY s.timestamp ASC
        "#;
        
        let search_term = format!("%{}%", topic);
        let mentions = sqlx::query(search_query)
            .bind(&search_term)
            .bind(&search_term)
            .fetch_all(&self.database.pool)
            .await?;
        
        // Group by date for trend analysis
        let mut daily_mentions = std::collections::HashMap::new();
        for mention in &mentions {
            let date = mention.get::<String, _>("discussion_date");
            let entry = daily_mentions.entry(date).or_insert_with(Vec::new);
            entry.push(json!({
                "text": mention.get::<String, _>("text"),
                "speaker": mention.get::<Option<String>, _>("speaker"),
                "conversation_title": mention.get::<Option<String>, _>("conversation_title"),
                "timestamp": mention.get::<chrono::DateTime<chrono::Utc>, _>("timestamp")
            }));
        }
        
        let chronological_data: Vec<Value> = daily_mentions
            .into_iter()
            .map(|(date, mentions)| {
                json!({
                    "date": date,
                    "mention_count": mentions.len(),
                    "mentions": mentions
                })
            })
            .collect();
        
        let prompt_template = format!(r#"Analyze the evolution and development of the topic "{}" over time:

Time Range: {}
Total Mentions Found: {}

Chronological Topic Data:
{}

Please provide an analysis including:
1. Topic evolution timeline and key milestones
2. Changes in how the topic is discussed over time
3. Different perspectives or approaches that emerged
4. Key contributors and their viewpoints
5. Trends and patterns in the discussions
6. Current status and future implications
7. Notable quotes or insights related to this topic

Format your response as a topic evolution analysis report."#,
            topic,
            time_range,
            mentions.len(),
            serde_json::to_string_pretty(&chronological_data)?
        );
        
        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_template
                    }
                }
            ],
            "metadata": {
                "topic": topic,
                "time_range": time_range,
                "total_mentions": mentions.len(),
                "date_range": chronological_data.len()
            }
        }))
    }
    
    /// Weekly summary prompt
    async fn prompt_weekly_summary(&self, args: &Value) -> Result<Value> {
        let week_offset = args.get("week_offset")
            .and_then(|w| w.as_i64())
            .unwrap_or(0);
            
        let include_metrics = args.get("include_metrics")
            .and_then(|m| m.as_bool())
            .unwrap_or(true);
        
        // Calculate week boundaries
        let start_date = format!("datetime('now', 'localtime', 'weekday 0', '-{} weeks', '-6 days')", week_offset);
        let end_date = format!("datetime('now', 'localtime', 'weekday 0', '-{} weeks')", week_offset);
        
        // Get week's conversations
        let conversations_query = format!(r#"
            SELECT c.*, COUNT(s.id) as segment_count,
                   COUNT(DISTINCT s.speaker) as unique_speakers,
                   SUM(s.end_time - s.start_time) as total_duration
            FROM conversations c
            LEFT JOIN segments s ON c.id = s.conversation_id
            WHERE c.start_time BETWEEN {} AND {}
            GROUP BY c.id
            ORDER BY c.start_time DESC
        "#, start_date, end_date);
        
        let conversations = sqlx::query(&conversations_query)
            .fetch_all(&self.database.pool)
            .await?;
        
        // Get speaker activity for the week
        let speaker_activity_query = format!(r#"
            SELECT s.speaker, 
                   COUNT(*) as segments,
                   SUM(s.end_time - s.start_time) as duration,
                   COUNT(DISTINCT s.conversation_id) as conversations
            FROM segments s
            JOIN conversations c ON s.conversation_id = c.id
            WHERE c.start_time BETWEEN {} AND {}
              AND s.speaker IS NOT NULL
            GROUP BY s.speaker
            ORDER BY duration DESC
        "#, start_date, end_date);
        
        let speaker_activity = sqlx::query(&speaker_activity_query)
            .fetch_all(&self.database.pool)
            .await?;
        
        let conversation_summaries: Vec<Value> = conversations
            .iter()
            .map(|conv| {
                json!({
                    "title": conv.get::<Option<String>, _>("title"),
                    "start_time": conv.get::<chrono::DateTime<chrono::Utc>, _>("start_time"),
                    "segment_count": conv.get::<i64, _>("segment_count"),
                    "unique_speakers": conv.get::<i64, _>("unique_speakers"),
                    "duration": conv.get::<Option<f64>, _>("total_duration")
                })
            })
            .collect();
        
        let speaker_summaries: Vec<Value> = speaker_activity
            .iter()
            .map(|activity| {
                json!({
                    "speaker": activity.get::<String, _>("speaker"),
                    "segments": activity.get::<i64, _>("segments"),
                    "duration": activity.get::<f64, _>("duration"),
                    "conversations": activity.get::<i64, _>("conversations")
                })
            })
            .collect();
        
        let week_description = match week_offset {
            0 => "This week",
            1 => "Last week",
            n => &format!("{} weeks ago", n)
        };
        
        let metrics_section = if include_metrics {
            format!(r#"

Weekly Metrics:
- Total Conversations: {}
- Total Participants: {}
- Most Active Speaker: {}
- Total Discussion Time: {:.1} hours

Conversation Details:
{}

Speaker Activity:
{}"#,
                conversations.len(),
                speaker_activity.len(),
                speaker_summaries.first()
                    .and_then(|s| s.get("speaker"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("None"),
                conversation_summaries.iter()
                    .map(|c| c.get("duration").and_then(|d| d.as_f64()).unwrap_or(0.0))
                    .sum::<f64>() / 3600.0,
                serde_json::to_string_pretty(&conversation_summaries)?,
                serde_json::to_string_pretty(&speaker_summaries)?
            )
        } else {
            String::new()
        };
        
        let prompt_template = format!(r#"Generate a comprehensive weekly summary for {}:

{}{} 

Please provide:
1. Executive Summary (2-3 sentences overview)
2. Key Highlights and Achievements
3. Important Discussions and Decisions
4. Participant Insights and Contributions
5. Emerging Themes or Patterns
6. Action Items and Follow-ups
7. Recommendations for Next Week

Format your response as a professional weekly summary report."#,
            week_description,
            if conversations.is_empty() { "No conversations found for this week." } else { "Weekly activity data:" },
            metrics_section
        );
        
        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_template
                    }
                }
            ],
            "metadata": {
                "week_offset": week_offset,
                "include_metrics": include_metrics,
                "conversation_count": conversations.len(),
                "speaker_count": speaker_activity.len()
            }
        }))
    }
    
    /// Meeting preparation prompt
    async fn prompt_meeting_preparation(&self, args: &Value) -> Result<Value> {
        let participants_str = args.get("participants")
            .and_then(|p| p.as_str())
            .ok_or_else(|| anyhow!("Missing participants parameter"))?;
            
        let participants: Vec<&str> = participants_str.split(',').map(|p| p.trim()).collect();
        let topics = args.get("topics")
            .and_then(|t| t.as_str())
            .unwrap_or("");
        
        // Get recent conversations involving these participants
        let placeholders = participants.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let recent_conversations_query = format!(r#"
            SELECT DISTINCT c.id, c.title, c.start_time, c.context,
                   GROUP_CONCAT(DISTINCT s.speaker) as all_speakers
            FROM conversations c
            JOIN segments s ON c.id = s.conversation_id
            WHERE s.speaker IN ({})
              AND c.start_time > datetime('now', '-30 days')
            GROUP BY c.id
            HAVING COUNT(DISTINCT s.speaker) >= 2
            ORDER BY c.start_time DESC
            LIMIT 10
        "#, placeholders);
        
        let mut query_builder = sqlx::query(&recent_conversations_query);
        for participant in &participants {
            query_builder = query_builder.bind(participant);
        }
        
        let recent_conversations = query_builder.fetch_all(&self.database.pool).await?;
        
        // Get individual participant context
        let mut participant_context = Vec::new();
        for participant in &participants {
            let analytics_query = r#"
                SELECT * FROM speaker_analytics WHERE speaker = ?
            "#;
            
            if let Ok(Some(analytics)) = sqlx::query(analytics_query)
                .bind(participant)
                .fetch_optional(&self.database.pool)
                .await 
            {
                // Get recent topics for this participant
                let topics_query = r#"
                    SELECT s.text, s.timestamp, c.title as conversation_title
                    FROM segments s
                    JOIN conversations c ON s.conversation_id = c.id
                    WHERE s.speaker = ?
                      AND s.timestamp > datetime('now', '-7 days')
                    ORDER BY s.timestamp DESC
                    LIMIT 5
                "#;
                
                let recent_topics = sqlx::query(topics_query)
                    .bind(participant)
                    .fetch_all(&self.database.pool)
                    .await
                    .unwrap_or_default();
                
                participant_context.push(json!({
                    "participant": participant,
                    "total_conversations": analytics.get::<i64, _>("conversation_count"),
                    "recent_activity": recent_topics.len(),
                    "communication_style": "collaborative", // This would be derived from analysis
                    "recent_topics": recent_topics.iter().map(|row| {
                        row.get::<String, _>("text")
                    }).collect::<Vec<_>>()
                }));
            }
        }
        
        let conversation_summaries: Vec<Value> = recent_conversations
            .iter()
            .map(|conv| {
                json!({
                    "title": conv.get::<Option<String>, _>("title"),
                    "date": conv.get::<chrono::DateTime<chrono::Utc>, _>("start_time"),
                    "participants": conv.get::<String, _>("all_speakers"),
                    "context": conv.get::<Option<String>, _>("context")
                })
            })
            .collect();
        
        let prompt_template = format!(r#"Prepare a comprehensive briefing for an upcoming meeting with these participants:

Participants: {}
Expected Topics: {}

Recent Group Conversations (last 30 days):
{}

Individual Participant Context:
{}

Please provide a meeting preparation briefing including:
1. Participant Background and Communication Styles
2. Recent Discussion History and Context
3. Potential Areas of Agreement and Disagreement
4. Relevant Previous Decisions or Commitments
5. Suggested Discussion Topics and Questions
6. Potential Challenges and How to Address Them
7. Recommended Meeting Structure and Flow
8. Key Points to Remember for Each Participant

Format your response as a professional meeting preparation briefing."#,
            participants_str,
            if topics.is_empty() { "Not specified" } else { topics },
            serde_json::to_string_pretty(&conversation_summaries)?,
            serde_json::to_string_pretty(&participant_context)?
        );
        
        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_template
                    }
                }
            ],
            "metadata": {
                "participants": participants,
                "participant_count": participants.len(),
                "recent_conversations": recent_conversations.len(),
                "topics": topics
            }
        }))
    }
    
    /// Action items tracker prompt
    async fn prompt_action_items_tracker(&self, args: &Value) -> Result<Value> {
        let speaker = args.get("speaker")
            .and_then(|s| s.as_str());
            
        let status = args.get("status")
            .and_then(|s| s.as_str())
            .unwrap_or("all");
        
        // Search for action item patterns in conversations
        let action_patterns = vec![
            "I will",
            "I'll",
            "action item",
            "TODO",
            "follow up",
            "next step",
            "by next week",
            "assigned to",
            "responsible for"
        ];
        
        let mut action_items = Vec::new();
        
        for pattern in &action_patterns {
            let search_query = if let Some(_speaker_name) = speaker {
                r#"
                    SELECT s.text, s.speaker, s.timestamp, c.title as conversation_title, c.id as conversation_id
                    FROM segments s
                    JOIN conversations c ON s.conversation_id = c.id
                    WHERE (s.text LIKE ? OR s.processed_text LIKE ?)
                      AND s.speaker = ?
                      AND s.timestamp > datetime('now', '-60 days')
                    ORDER BY s.timestamp DESC
                    LIMIT 20
                "#
            } else {
                r#"
                    SELECT s.text, s.speaker, s.timestamp, c.title as conversation_title, c.id as conversation_id
                    FROM segments s
                    JOIN conversations c ON s.conversation_id = c.id
                    WHERE (s.text LIKE ? OR s.processed_text LIKE ?)
                      AND s.timestamp > datetime('now', '-60 days')
                    ORDER BY s.timestamp DESC
                    LIMIT 20
                "#
            };
            
            let search_term = format!("%{}%", pattern);
            let mut query_builder = sqlx::query(search_query)
                .bind(&search_term)
                .bind(&search_term);
                
            if let Some(speaker_name) = speaker {
                query_builder = query_builder.bind(speaker_name);
            }
            
            if let Ok(results) = query_builder.fetch_all(&self.database.pool).await {
                for result in results {
                    action_items.push(json!({
                        "text": result.get::<String, _>("text"),
                        "speaker": result.get::<Option<String>, _>("speaker"),
                        "timestamp": result.get::<chrono::DateTime<chrono::Utc>, _>("timestamp"),
                        "conversation_title": result.get::<Option<String>, _>("conversation_title"),
                        "conversation_id": result.get::<String, _>("conversation_id"),
                        "pattern": pattern,
                        "estimated_status": "pending" // This would be enhanced with NLP analysis
                    }));
                }
            }
        }
        
        // Sort by timestamp (most recent first)
        action_items.sort_by(|a, b| {
            let timestamp_a = a.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
            let timestamp_b = b.get("timestamp").and_then(|t| t.as_str()).unwrap_or("");
            timestamp_b.cmp(timestamp_a)
        });
        
        // Take top 50 most recent
        action_items.truncate(50);
        
        let filter_description = match (speaker, status) {
            (Some(s), "all") => format!("for speaker: {}", s),
            (None, "all") => "for all speakers".to_string(),
            (Some(s), st) => format!("for speaker: {} with status: {}", s, st),
            (None, st) => format!("with status: {}", st),
        };
        
        let prompt_template = format!(r#"Analyze and organize the following potential action items and commitments {}:

Potential Action Items Found:
{}

Please provide an organized action items report including:
1. Clearly Identified Action Items (extract specific commitments)
2. Responsible Parties (who is assigned to each item)
3. Deadlines or Time Frames (if mentioned)
4. Priority Assessment (high, medium, low)
5. Current Status Evaluation (completed, in progress, pending, overdue)
6. Follow-up Requirements
7. Related Context and Background
8. Recommendations for Tracking and Management

Note: Some items may be informal mentions rather than formal commitments. Please distinguish between firm action items and casual references.

Format your response as a structured action items tracking report."#,
            filter_description,
            serde_json::to_string_pretty(&action_items)?
        );
        
        Ok(json!({
            "messages": [
                {
                    "role": "user",
                    "content": {
                        "type": "text",
                        "text": prompt_template
                    }
                }
            ],
            "metadata": {
                "speaker": speaker,
                "status": status,
                "total_items_found": action_items.len(),
                "search_patterns": action_patterns.len()
            }
        }))
    }
}