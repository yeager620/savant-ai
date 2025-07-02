//! Natural Language Query Interface Component
//! 
//! Provides a user-friendly interface for querying the conversation database with natural language

use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::console;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = ["window", "__TAURI__", "core"])]
    async fn invoke(cmd: &str, args: JsValue) -> JsValue;
}

/// Natural language query response from backend
#[derive(Debug, Clone, Serialize, Deserialize)]
struct QueryResponse {
    pub success: bool,
    pub results: serde_json::Value,
    pub summary: String,
    pub execution_time_ms: u64,
    pub intent_type: String,
    pub result_count: usize,
    pub error: Option<String>,
}

/// Database statistics response
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DatabaseStats {
    pub total_speakers: usize,
    pub total_conversations: i64,
    pub total_segments: i64,
    pub total_duration_hours: f64,
    pub average_conversation_length_minutes: f64,
    pub top_speakers: Vec<SpeakerSummary>,
}

/// Speaker summary for statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SpeakerSummary {
    pub name: String,
    pub conversation_count: i64,
    pub total_duration_hours: f64,
}

/// Search result item
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SearchResult {
    pub text: String,
    pub speaker: Option<String>,
    pub timestamp: String,
    pub confidence: Option<f64>,
    pub conversation_id: Option<String>,
}

/// Execute natural language query via Tauri
async fn execute_natural_query(query: String) -> Result<QueryResponse, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "query": query
    })).map_err(|e| e.to_string())?;
    
    let result = invoke("natural_language_query", args).await;
    let response: QueryResponse = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    Ok(response)
}

/// Get database statistics via Tauri
async fn get_database_stats() -> Result<DatabaseStats, String> {
    let result = invoke("get_database_stats", JsValue::NULL).await;
    let stats: DatabaseStats = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse stats: {}", e))?;
    
    Ok(stats)
}

/// Search conversations via Tauri
async fn search_conversations(query: String, limit: Option<usize>) -> Result<Vec<SearchResult>, String> {
    let args = serde_wasm_bindgen::to_value(&serde_json::json!({
        "query": query,
        "limit": limit
    })).map_err(|e| e.to_string())?;
    
    let result = invoke("search_conversations", args).await;
    let response: serde_json::Value = serde_wasm_bindgen::from_value(result)
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let results: Vec<SearchResult> = serde_json::from_value(
        response.get("results").unwrap_or(&serde_json::Value::Array(vec![])).clone()
    ).map_err(|e| format!("Failed to parse results: {}", e))?;
    
    Ok(results)
}

/// Main natural language query interface component
#[component]
pub fn NaturalQueryInterface() -> impl IntoView {
    let (query, set_query) = signal(String::new());
    let (loading, set_loading) = signal(false);
    let (response, set_response) = signal(None::<QueryResponse>);
    let (active_tab, set_active_tab) = signal("query");
    let (database_stats, set_database_stats) = signal(None::<DatabaseStats>);
    let (search_results, set_search_results) = signal(Vec::<SearchResult>::new());
    
    // Load database stats on mount
    let _effect = Effect::new(move |_| {
        spawn_local(async move {
            match get_database_stats().await {
                Ok(stats) => set_database_stats.set(Some(stats)),
                Err(e) => console::log_1(&format!("Failed to load database stats: {}", e).into()),
            }
        });
    });
    
    let execute_query = move || {
        let current_query = query.get();
        if current_query.trim().is_empty() {
            return;
        }
        
        set_loading.set(true);
        
        spawn_local(async move {
            match execute_natural_query(current_query).await {
                Ok(result) => {
                    set_response.set(Some(result));
                    set_loading.set(false);
                }
                Err(e) => {
                    console::log_1(&format!("Query failed: {}", e).into());
                    set_response.set(Some(QueryResponse {
                        success: false,
                        results: serde_json::Value::Null,
                        summary: format!("Error: {}", e),
                        execution_time_ms: 0,
                        intent_type: "error".to_string(),
                        result_count: 0,
                        error: Some(e),
                    }));
                    set_loading.set(false);
                }
            }
        });
    };
    
    let perform_search = move |search_query: String| {
        if search_query.trim().is_empty() {
            return;
        }
        
        spawn_local(async move {
            match search_conversations(search_query, Some(20)).await {
                Ok(results) => set_search_results.set(results),
                Err(e) => console::log_1(&format!("Search failed: {}", e).into()),
            }
        });
    };
    
    view! {
        <div class="natural-query-interface">
            <div class="tabs">
                <button 
                    class:active=move || active_tab.get() == "query"
                    on:click=move |_| set_active_tab("query")
                >
                    "Natural Language Query"
                </button>
                <button 
                    class:active=move || active_tab.get() == "search"
                    on:click=move |_| set_active_tab("search")
                >
                    "Search Conversations"
                </button>
                <button 
                    class:active=move || active_tab.get() == "stats"
                    on:click=move |_| set_active_tab("stats")
                >
                    "Database Statistics"
                </button>
            </div>
            
            <div class="tab-content">
                // Natural Language Query Tab
                <div class="tab-panel" class:active=move || active_tab.get() == "query">
                    <div class="query-section">
                        <h3>"Ask your database anything in natural language"</h3>
                        <div class="query-examples">
                            <p>"Try asking:"</p>
                            <ul>
                                <li>"Find all conversations with John from last week"</li>
                                <li>"How much did Alice talk yesterday?"</li>
                                <li>"Search for mentions of project alpha"</li>
                                <li>"Show me database statistics"</li>
                                <li>"Who are the most active speakers?"</li>
                            </ul>
                        </div>
                        
                        <div class="query-input">
                            <textarea
                                placeholder="Ask anything about your conversations..."
                                value=move || query.get()
                                on:input=move |ev| set_query.set(event_target_value(&ev))
                                on:keydown=move |ev| {
                                    if ev.key() == "Enter" && !ev.shift_key() {
                                        ev.prevent_default();
                                        execute_query();
                                    }
                                }
                                disabled=loading
                            />
                            <button
                                on:click=move |_| execute_query()
                                disabled=move || loading.get() || query.get().trim().is_empty()
                                class="query-button"
                            >
                                {move || if loading.get() { "Thinking..." } else { "Ask" }}
                            </button>
                        </div>
                        
                        {move || {
                            response.get().map(|resp| {
                                view! {
                                    <div class="query-results">
                                        <div class="result-header">
                                            <h4>{&resp.summary}</h4>
                                            <div class="result-meta">
                                                <span class="intent">"Intent: " {&resp.intent_type}</span>
                                                <span class="timing">"Time: " {resp.execution_time_ms} "ms"</span>
                                                <span class="count">"Results: " {resp.result_count}</span>
                                            </div>
                                        </div>
                                        
                                        {if resp.success {
                                            view! {
                                                <div class="results-content">
                                                    <QueryResultsDisplay results=resp.results.clone() intent_type=resp.intent_type.clone() />
                                                </div>
                                            }
                                        } else {
                                            view! {
                                                <div class="error-content">
                                                    <p class="error">{resp.error.unwrap_or_else(|| "Unknown error".to_string())}</p>
                                                </div>
                                            }
                                        }}
                                    </div>
                                }
                            })
                        }}
                    </div>
                </div>
                
                // Search Tab
                <div class="tab-panel" class:active=move || active_tab.get() == "search">
                    <div class="search-section">
                        <h3>"Search conversation content"</h3>
                        <SearchInterface on_search=perform_search results=search_results />
                    </div>
                </div>
                
                // Statistics Tab
                <div class="tab-panel" class:active=move || active_tab.get() == "stats">
                    <div class="stats-section">
                        <h3>"Database Overview"</h3>
                        {move || {
                            database_stats.get().map(|stats| {
                                view! {
                                    <DatabaseStatsDisplay stats=stats />
                                }
                            }).unwrap_or_else(|| {
                                view! {
                                    <div class="loading">"Loading statistics..."</div>
                                }
                            })
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

/// Component for displaying query results based on intent type
#[component]
fn QueryResultsDisplay(
    results: serde_json::Value,
    intent_type: String,
) -> impl IntoView {
    match intent_type.as_str() {
        "find_conversations" => {
            view! {
                <div class="conversations-results">
                    {move || {
                        if let Some(array) = results.as_array() {
                            array.iter().map(|item| {
                                let title = item.get("title").and_then(|t| t.as_str()).unwrap_or("Untitled");
                                let participants = item.get("participants").and_then(|p| p.as_array())
                                    .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>().join(", "))
                                    .unwrap_or_else(|| "Unknown".to_string());
                                let duration = item.get("total_duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
                                
                                view! {
                                    <div class="conversation-item">
                                        <h5>{title}</h5>
                                        <p>"Participants: " {participants}</p>
                                        <p>"Duration: " {format!("{:.1} minutes", duration / 60.0)}</p>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        } else {
                            vec![view! { <p>"No conversations found"</p> }]
                        }
                    }}
                </div>
            }
        }
        "analyze_speaker" => {
            view! {
                <div class="speaker-analysis">
                    {move || {
                        if let Some(array) = results.as_array() {
                            array.iter().map(|item| {
                                let speaker = item.get("speaker").and_then(|s| s.as_str()).unwrap_or("Unknown");
                                let conv_count = item.get("conversation_count").and_then(|c| c.as_i64()).unwrap_or(0);
                                let duration = item.get("total_duration").and_then(|d| d.as_f64()).unwrap_or(0.0);
                                let confidence = item.get("avg_confidence").and_then(|c| c.as_f64()).unwrap_or(0.0);
                                
                                view! {
                                    <div class="speaker-stats">
                                        <h5>{speaker}</h5>
                                        <div class="stats-grid">
                                            <div class="stat">
                                                <label>"Conversations:"</label>
                                                <span>{conv_count}</span>
                                            </div>
                                            <div class="stat">
                                                <label>"Total Time:"</label>
                                                <span>{format!("{:.1} hours", duration / 3600.0)}</span>
                                            </div>
                                            <div class="stat">
                                                <label>"Avg Confidence:"</label>
                                                <span>{format!("{:.1}%", confidence * 100.0)}</span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect::<Vec<_>>()
                        } else {
                            vec![view! { <p>"No speaker data found"</p> }]
                        }
                    }}
                </div>
            }
        }
        _ => {
            view! {
                <div class="generic-results">
                    <pre>{serde_json::to_string_pretty(&results).unwrap_or_else(|_| "Invalid JSON".to_string())}</pre>
                </div>
            }
        }
    }
}

/// Search interface component
#[component]
fn SearchInterface<F>(
    on_search: F,
    results: ReadSignal<Vec<SearchResult>>,
) -> impl IntoView 
where
    F: Fn(String) + 'static + Clone,
{
    let (search_query, set_search_query) = signal(String::new());
    
    let perform_search = move || {
        let query = search_query.get();
        if !query.trim().is_empty() {
            on_search(query);
        }
    };
    
    view! {
        <div class="search-interface">
            <div class="search-input">
                <input
                    type="text"
                    placeholder="Search conversation content..."
                    value=move || search_query.get()
                    on:input=move |ev| set_search_query.set(event_target_value(&ev))
                    on:keydown=move |ev| {
                        if ev.key() == "Enter" {
                            perform_search();
                        }
                    }
                />
                <button on:click=move |_| perform_search()>"Search"</button>
            </div>
            
            <div class="search-results">
                {move || {
                    let results_vec = results.get();
                    if results_vec.is_empty() {
                        view! {
                            <p class="no-results">"No search results yet. Try searching for something!"</p>
                        }
                    } else {
                        view! {
                            <div class="results-list">
                                {results_vec.into_iter().map(|result| {
                                    view! {
                                        <div class="search-result-item">
                                            <div class="result-header">
                                                <span class="speaker">{result.speaker.unwrap_or_else(|| "Unknown".to_string())}</span>
                                                <span class="timestamp">{result.timestamp}</span>
                                            </div>
                                            <p class="result-text">{result.text}</p>
                                            {result.confidence.map(|conf| {
                                                view! {
                                                    <div class="confidence">
                                                        "Confidence: " {format!("{:.1}%", conf * 100.0)}
                                                    </div>
                                                }
                                            })}
                                        </div>
                                    }
                                }).collect::<Vec<_>>()}
                            </div>
                        }
                    }
                }}
            </div>
        </div>
    }
}

/// Database statistics display component
#[component]
fn DatabaseStatsDisplay(stats: DatabaseStats) -> impl IntoView {
    view! {
        <div class="database-stats">
            <div class="stats-overview">
                <div class="stat-card">
                    <h4>"Total Conversations"</h4>
                    <div class="stat-value">{stats.total_conversations}</div>
                </div>
                <div class="stat-card">
                    <h4>"Unique Speakers"</h4>
                    <div class="stat-value">{stats.total_speakers}</div>
                </div>
                <div class="stat-card">
                    <h4>"Total Segments"</h4>
                    <div class="stat-value">{stats.total_segments}</div>
                </div>
                <div class="stat-card">
                    <h4>"Total Duration"</h4>
                    <div class="stat-value">{format!("{:.1}h", stats.total_duration_hours)}</div>
                </div>
            </div>
            
            <div class="top-speakers">
                <h4>"Most Active Speakers"</h4>
                <div class="speakers-list">
                    {stats.top_speakers.into_iter().map(|speaker| {
                        view! {
                            <div class="speaker-item">
                                <span class="speaker-name">{speaker.name}</span>
                                <span class="speaker-stats">
                                    {speaker.conversation_count} " conversations, " 
                                    {format!("{:.1}h", speaker.total_duration_hours)}
                                </span>
                            </div>
                        }
                    }).collect::<Vec<_>>()}
                </div>
            </div>
        </div>
    }
}