use leptos::*;
use serde::{Deserialize, Serialize};
use savant_video::GeneratedSolution;
use savant_video::DetectedCodingProblem;
use crate::commands;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SolutionState {
    pub visible: bool,
    pub current_problem: Option<DetectedCodingProblem>,
    pub current_solution: Option<GeneratedSolution>,
    pub position: Position,
    pub minimized: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
}

impl Default for SolutionState {
    fn default() -> Self {
        Self {
            visible: false,
            current_problem: None,
            current_solution: None,
            position: Position {
                x: 20.0,  // Right side of screen
                y: 100.0, // Below menu bar
            },
            minimized: false,
        }
    }
}

#[component]
pub fn SolutionDisplay() -> impl IntoView {
    let (solution_state, set_solution_state) = create_signal(SolutionState::default());
    let (is_dragging, set_is_dragging) = create_signal(false);
    let (drag_start, set_drag_start) = create_signal((0.0, 0.0));

    // Listen for new solutions via Tauri events
    create_effect(move |_| {
        spawn_local(async move {
            let _ = commands::listen_for_solutions(move |problem, solution| {
                set_solution_state.update(|state| {
                    state.current_problem = Some(problem);
                    state.current_solution = Some(solution);
                    state.visible = true;
                    state.minimized = false;
                });
            }).await;
        });
    });

    // Handle window dragging
    let handle_mouse_down = move |e: web_sys::MouseEvent| {
        set_is_dragging(true);
        set_drag_start((e.client_x() as f32, e.client_y() as f32));
        e.prevent_default();
    };

    let handle_mouse_move = move |e: web_sys::MouseEvent| {
        if is_dragging() {
            let (start_x, start_y) = drag_start();
            let delta_x = e.client_x() as f32 - start_x;
            let delta_y = e.client_y() as f32 - start_y;
            
            set_solution_state.update(|state| {
                state.position.x += delta_x;
                state.position.y += delta_y;
            });
            
            set_drag_start((e.client_x() as f32, e.client_y() as f32));
        }
    };

    let handle_mouse_up = move |_| {
        set_is_dragging(false);
    };

    // Add global mouse listeners
    create_effect(move |_| {
        if is_dragging() {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            
            let mousemove_closure = Closure::<dyn Fn(_)>::new(move |e: web_sys::MouseEvent| {
                handle_mouse_move(e);
            });
            
            let mouseup_closure = Closure::<dyn Fn(_)>::new(move |e: web_sys::MouseEvent| {
                handle_mouse_up(e);
            });
            
            document.add_event_listener_with_callback(
                "mousemove",
                mousemove_closure.as_ref().unchecked_ref()
            ).unwrap();
            
            document.add_event_listener_with_callback(
                "mouseup",
                mouseup_closure.as_ref().unchecked_ref()
            ).unwrap();
            
            // Store closures to prevent them from being dropped
            mousemove_closure.forget();
            mouseup_closure.forget();
        }
    });

    view! {
        <Show when=move || solution_state().visible>
            <div
                class="solution-overlay"
                style=move || format!(
                    "left: {}px; top: {}px; {}",
                    solution_state().position.x,
                    solution_state().position.y,
                    if solution_state().minimized { "height: 40px;" } else { "" }
                )
            >
                // Title bar
                <div 
                    class="solution-titlebar"
                    on:mousedown=handle_mouse_down
                >
                    <div class="solution-title">
                        {move || {
                            solution_state().current_problem
                                .as_ref()
                                .map(|p| p.title.clone())
                                .unwrap_or_else(|| "Solution Assistant".to_string())
                        }}
                    </div>
                    <div class="solution-controls">
                        <button
                            class="solution-control-btn"
                            on:click=move |_| {
                                set_solution_state.update(|state| {
                                    state.minimized = !state.minimized;
                                });
                            }
                        >
                            {move || if solution_state().minimized { "▲" } else { "▼" }}
                        </button>
                        <button
                            class="solution-control-btn"
                            on:click=move |_| {
                                set_solution_state.update(|state| {
                                    state.visible = false;
                                });
                            }
                        >
                            "✕"
                        </button>
                    </div>
                </div>

                // Content area
                <Show when=move || !solution_state().minimized>
                    <div class="solution-content">
                        // Problem description
                        <Show when=move || solution_state().current_problem.is_some()>
                            <div class="solution-section">
                                <h3 class="solution-section-title">"Problem Detected"</h3>
                                <div class="problem-type">
                                    {move || {
                                        solution_state().current_problem
                                            .as_ref()
                                            .map(|p| format!("{:?}", p.problem_type))
                                            .unwrap_or_default()
                                    }}
                                </div>
                                <div class="problem-description">
                                    {move || {
                                        solution_state().current_problem
                                            .as_ref()
                                            .map(|p| p.description.clone())
                                            .unwrap_or_default()
                                    }}
                                </div>
                            </div>
                        </Show>

                        // Solution code
                        <Show when=move || solution_state().current_solution.is_some()>
                            <div class="solution-section">
                                <h3 class="solution-section-title">
                                    "Generated Solution"
                                    <span class="solution-confidence">
                                        {move || {
                                            solution_state().current_solution
                                                .as_ref()
                                                .map(|s| format!("{}% confidence", (s.confidence_score * 100.0) as i32))
                                                .unwrap_or_default()
                                        }}
                                    </span>
                                </h3>
                                <div class="solution-code-wrapper">
                                    <pre class="solution-code">
                                        <code>
                                            {move || {
                                                solution_state().current_solution
                                                    .as_ref()
                                                    .map(|s| s.solution_code.clone())
                                                    .unwrap_or_default()
                                            }}
                                        </code>
                                    </pre>
                                    <button
                                        class="copy-button"
                                        on:click=move |_| {
                                            if let Some(solution) = solution_state().current_solution.as_ref() {
                                                let _ = commands::copy_to_clipboard(&solution.solution_code);
                                            }
                                        }
                                    >
                                        "📋 Copy"
                                    </button>
                                </div>
                            </div>

                            // Explanation
                            <Show when=move || {
                                solution_state().current_solution
                                    .as_ref()
                                    .and_then(|s| s.explanation.as_ref())
                                    .is_some()
                            }>
                                <div class="solution-section">
                                    <h3 class="solution-section-title">"Explanation"</h3>
                                    <div class="solution-explanation">
                                        {move || {
                                            solution_state().current_solution
                                                .as_ref()
                                                .and_then(|s| s.explanation.clone())
                                                .unwrap_or_default()
                                        }}
                                    </div>
                                </div>
                            </Show>

                            // Complexity analysis
                            <Show when=move || {
                                let sol = solution_state().current_solution.as_ref();
                                sol.and_then(|s| s.time_complexity.as_ref()).is_some() ||
                                sol.and_then(|s| s.space_complexity.as_ref()).is_some()
                            }>
                                <div class="solution-section complexity-section">
                                    <h3 class="solution-section-title">"Complexity Analysis"</h3>
                                    <div class="complexity-grid">
                                        <Show when=move || {
                                            solution_state().current_solution
                                                .as_ref()
                                                .and_then(|s| s.time_complexity.as_ref())
                                                .is_some()
                                        }>
                                            <div class="complexity-item">
                                                <strong>"Time:"</strong>
                                                {move || {
                                                    solution_state().current_solution
                                                        .as_ref()
                                                        .and_then(|s| s.time_complexity.clone())
                                                        .unwrap_or_default()
                                                }}
                                            </div>
                                        </Show>
                                        <Show when=move || {
                                            solution_state().current_solution
                                                .as_ref()
                                                .and_then(|s| s.space_complexity.as_ref())
                                                .is_some()
                                        }>
                                            <div class="complexity-item">
                                                <strong>"Space:"</strong>
                                                {move || {
                                                    solution_state().current_solution
                                                        .as_ref()
                                                        .and_then(|s| s.space_complexity.clone())
                                                        .unwrap_or_default()
                                                }}
                                            </div>
                                        </Show>
                                    </div>
                                </div>
                            </Show>

                            // Test results
                            <Show when=move || {
                                solution_state().current_solution
                                    .as_ref()
                                    .map(|s| !s.test_results.is_empty())
                                    .unwrap_or(false)
                            }>
                                <div class="solution-section">
                                    <h3 class="solution-section-title">"Test Results"</h3>
                                    <div class="test-results">
                                        {move || {
                                            solution_state().current_solution
                                                .as_ref()
                                                .map(|s| {
                                                    let passed = s.test_results.iter().filter(|t| t.passed).count();
                                                    let total = s.test_results.len();
                                                    format!("{}/{} tests passed", passed, total)
                                                })
                                                .unwrap_or_default()
                                        }}
                                    </div>
                                </div>
                            </Show>
                        </Show>

                        // Action buttons
                        <div class="solution-actions">
                            <button
                                class="solution-action-btn primary"
                                on:click=move |_| {
                                    if let Some(solution) = solution_state().current_solution.as_ref() {
                                        spawn_local(async move {
                                            let _ = commands::apply_solution(&solution.solution_code).await;
                                        });
                                    }
                                }
                            >
                                "Apply Solution"
                            </button>
                            <button
                                class="solution-action-btn"
                                on:click=move |_| {
                                    spawn_local(async move {
                                        let _ = commands::regenerate_solution().await;
                                    });
                                }
                            >
                                "Regenerate"
                            </button>
                            <button
                                class="solution-action-btn"
                                on:click=move |_| {
                                    set_solution_state.update(|state| {
                                        state.visible = false;
                                    });
                                }
                            >
                                "Dismiss"
                            </button>
                        </div>
                    </div>
                </Show>
            </div>
        </Show>
    }
}