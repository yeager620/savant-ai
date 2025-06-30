use leptos::prelude::*;
use crate::components::{Dashboard, Overlay};

#[component]
pub fn App() -> impl IntoView {
    let (show_overlay, set_show_overlay) = signal(true);

    view! {
        <div class="app">
            <div class="overlay-layer" class:hidden=move || !show_overlay.get()>
                <Overlay />
            </div>

            <div class="main-view">
                <div class="dashboard-view">
                    <Dashboard />
                    <div class="view-controls">
                        <button 
                            class="overlay-toggle"
                            on:click=move |_| {
                                let new_state = !show_overlay.get();
                                set_show_overlay.set(new_state);
                            }
                        >
                            {move || if show_overlay.get() { "Hide Overlay" } else { "Show Overlay" }}
                        </button>
                    </div>
                </div>
            </div>
            
            // Global CSS styles
            <style>
                "
                * {
                    margin: 0;
                    padding: 0;
                    box-sizing: border-box;
                }

                body {
                    font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
                    background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
                    color: white;
                    min-height: 100vh;
                }

                .app {
                    width: 100vw;
                    min-height: 100vh;
                    position: relative;
                }

                .overlay-layer {
                    position: fixed;
                    top: 20px;
                    right: 20px;
                    z-index: 9999;
                    pointer-events: none;
                }

                .hidden {
                    display: none !important;
                }

                .overlay-layer * {
                    pointer-events: auto;
                }

                .main-view {
                    width: 100%;
                    min-height: 100vh;
                    padding: 20px;
                }

                .dashboard-view {
                    max-width: 1200px;
                    margin: 0 auto;
                }

                .view-controls {
                    margin-top: 20px;
                    text-align: center;
                }

                .overlay-toggle, .dashboard-btn {
                    background: rgba(255, 255, 255, 0.2);
                    border: 1px solid rgba(255, 255, 255, 0.3);
                    color: white;
                    padding: 10px 20px;
                    border-radius: 8px;
                    cursor: pointer;
                    font-size: 14px;
                    transition: all 0.3s ease;
                }

                .overlay-toggle:hover, .dashboard-btn:hover {
                    background: rgba(255, 255, 255, 0.3);
                    transform: translateY(-2px);
                }

                /* Dashboard Styles */
                .dashboard {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 12px;
                    padding: 24px;
                    backdrop-filter: blur(10px);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }

                .dashboard-header {
                    text-align: center;
                    margin-bottom: 30px;
                }

                .dashboard-header h1 {
                    margin: 0 0 8px 0;
                    font-size: 28px;
                    font-weight: 600;
                }

                .dashboard-header p {
                    margin: 0;
                    opacity: 0.8;
                    font-size: 16px;
                }

                .config-sections {
                    display: grid;
                    gap: 24px;
                    grid-template-columns: 1fr 1fr;
                }

                @media (max-width: 768px) {
                    .config-sections {
                        grid-template-columns: 1fr;
                    }
                }

                .config-section {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 8px;
                    padding: 20px;
                    border: 1px solid rgba(255, 255, 255, 0.1);
                }

                .config-section h2 {
                    margin: 0 0 16px 0;
                    font-size: 18px;
                    font-weight: 500;
                    color: #fff;
                }

                .form-group {
                    margin-bottom: 16px;
                }

                .form-group label {
                    display: block;
                    margin-bottom: 6px;
                    font-size: 14px;
                    font-weight: 500;
                    color: rgba(255, 255, 255, 0.9);
                }

                .form-group input, .form-group select {
                    width: 100%;
                    padding: 8px 12px;
                    border: 1px solid rgba(255, 255, 255, 0.3);
                    border-radius: 6px;
                    background: rgba(255, 255, 255, 0.1);
                    color: white;
                    font-size: 14px;
                }

                .form-group input::placeholder {
                    color: rgba(255, 255, 255, 0.5);
                }

                .form-group input[type='checkbox'] {
                    width: auto;
                    margin-right: 8px;
                }

                .form-group input[type='range'] {
                    margin-right: 10px;
                }

                .action-buttons {
                    display: flex;
                    gap: 12px;
                    justify-content: center;
                    margin-top: 30px;
                    grid-column: 1 / -1;
                }

                .save-btn, .reset-btn, .test-btn {
                    padding: 12px 24px;
                    border: none;
                    border-radius: 6px;
                    font-size: 14px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.3s ease;
                }

                .save-btn {
                    background: #4CAF50;
                    color: white;
                }

                .save-btn:hover {
                    background: #45a049;
                    transform: translateY(-2px);
                }

                .reset-btn {
                    background: #f44336;
                    color: white;
                }

                .reset-btn:hover {
                    background: #da190b;
                    transform: translateY(-2px);
                }

                .test-btn {
                    background: #2196F3;
                    color: white;
                }

                .test-btn:hover {
                    background: #0b7dda;
                    transform: translateY(-2px);
                }

                /* Overlay Styles */
                .overlay-container {
                    background: rgba(0, 0, 0, 0.8);
                    border-radius: 12px;
                    padding: 16px;
                    max-width: 300px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
                }

                .overlay-status {
                    text-align: center;
                    margin-bottom: 12px;
                }

                .overlay-status p {
                    margin: 4px 0;
                    font-size: 14px;
                }

                .status-text {
                    font-size: 12px !important;
                    opacity: 0.8;
                }

                .overlay-toggle {
                    width: 100%;
                    margin-top: 8px;
                }

                .demo-bubble {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 8px;
                    padding: 12px;
                    margin-top: 12px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }

                .bubble-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 8px;
                }

                .question-preview {
                    font-size: 12px;
                    font-style: italic;
                    opacity: 0.8;
                }

                .close-btn {
                    background: none;
                    border: none;
                    color: white;
                    cursor: pointer;
                    font-size: 16px;
                    padding: 0;
                    width: 20px;
                    height: 20px;
                }

                .bubble-content {
                    font-size: 13px;
                    line-height: 1.4;
                }

                .bubble-content small {
                    opacity: 0.7;
                    font-size: 11px;
                }
                "
            </style>
        </div>
    }
}
