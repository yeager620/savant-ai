use leptos::prelude::*;
use crate::components::{Dashboard, Overlay, BrowserOverlay, MinimalChat};

#[derive(Debug, Clone, PartialEq)]
enum AppMode {
    VanillaChat,
    BrowserMonitoring,
}

#[component]
pub fn App() -> impl IntoView {
    let (show_overlay, set_show_overlay) = signal(true);
    let (app_mode, set_app_mode) = signal(AppMode::VanillaChat);

    view! {
        <div class="app">
            <div class="mode-selector">
                <button 
                    class="mode-btn"
                    class:active=move || app_mode.get() == AppMode::VanillaChat
                    on:click=move |_| set_app_mode.set(AppMode::VanillaChat)
                >
                    "💬 Vanilla Chat"
                </button>
                <button 
                    class="mode-btn"
                    class:active=move || app_mode.get() == AppMode::BrowserMonitoring
                    on:click=move |_| set_app_mode.set(AppMode::BrowserMonitoring)
                >
                    "🌐 Browser Assistant"
                </button>
            </div>

            <Show when=move || app_mode.get() == AppMode::VanillaChat>
                <div class="vanilla-chat-mode">
                    <div class="overlay-layer" class:hidden=move || !show_overlay.get()>
                        <Overlay />
                    </div>

                    <div class="main-view">
                        <div class="dashboard-view">
                            <Dashboard />
                            <div class="chat-interface">
                                <MinimalChat />
                            </div>
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
                </div>
            </Show>

            <Show when=move || app_mode.get() == AppMode::BrowserMonitoring>
                <div class="browser-monitoring-mode">
                    <div class="browser-interface">
                        <BrowserOverlay />
                    </div>
                    <div class="chat-interface">
                        <MinimalChat />
                    </div>
                </div>
            </Show>
            
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

                /* Mode Selector Styles */
                .mode-selector {
                    position: fixed;
                    top: 20px;
                    left: 50%;
                    transform: translateX(-50%);
                    z-index: 10000;
                    display: flex;
                    gap: 10px;
                    background: rgba(0, 0, 0, 0.8);
                    padding: 10px;
                    border-radius: 12px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    backdrop-filter: blur(10px);
                }

                .mode-btn {
                    background: rgba(255, 255, 255, 0.1);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    color: white;
                    padding: 8px 16px;
                    border-radius: 8px;
                    cursor: pointer;
                    font-size: 14px;
                    transition: all 0.3s ease;
                }

                .mode-btn:hover {
                    background: rgba(255, 255, 255, 0.2);
                    transform: translateY(-1px);
                }

                .mode-btn.active {
                    background: #4CAF50;
                    border-color: #4CAF50;
                    box-shadow: 0 2px 8px rgba(76, 175, 80, 0.3);
                }

                /* Browser Monitoring Mode Styles */
                .browser-monitoring-mode {
                    display: flex;
                    height: 100vh;
                    padding-top: 80px;
                }

                .browser-interface {
                    width: 50%;
                    padding: 20px;
                    border-right: 1px solid rgba(255, 255, 255, 0.2);
                    overflow-y: auto;
                }

                .chat-interface {
                    flex: 1;
                    padding: 20px;
                    overflow-y: auto;
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

                /* Browser Overlay Styles */
                .browser-overlay {
                    max-width: 500px;
                    margin: 0 auto;
                }

                .browser-controls {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 12px;
                    padding: 20px;
                    margin-bottom: 20px;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }

                .control-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    margin-bottom: 15px;
                }

                .control-header h3 {
                    margin: 0;
                    font-size: 18px;
                }

                .connection-status {
                    font-size: 12px;
                    padding: 4px 8px;
                    border-radius: 4px;
                    background: rgba(255, 0, 0, 0.2);
                }

                .connection-status.connected {
                    background: rgba(0, 255, 0, 0.2);
                }

                .control-buttons {
                    display: flex;
                    gap: 10px;
                    margin-bottom: 15px;
                }

                .start-btn, .stop-btn {
                    flex: 1;
                    padding: 10px;
                    border: none;
                    border-radius: 6px;
                    cursor: pointer;
                    font-size: 14px;
                    transition: all 0.3s ease;
                }

                .start-btn {
                    background: #4CAF50;
                    color: white;
                }

                .start-btn:disabled {
                    background: #666;
                    cursor: not-allowed;
                }

                .stop-btn {
                    background: #f44336;
                    color: white;
                }

                .stop-btn:disabled {
                    background: #666;
                    cursor: not-allowed;
                }

                .status-message {
                    font-size: 12px;
                    opacity: 0.8;
                    text-align: center;
                }

                .prompt-selection-overlay {
                    background: rgba(0, 0, 0, 0.9);
                    border-radius: 12px;
                    padding: 20px;
                    border: 2px solid #00ff41;
                    box-shadow: 0 4px 20px rgba(0, 255, 65, 0.3);
                }

                .prompt-header {
                    text-align: center;
                    margin-bottom: 20px;
                }

                .prompt-header h4 {
                    margin: 0 0 8px 0;
                    color: #00ff41;
                    font-size: 16px;
                }

                .navigation-hint {
                    font-size: 12px;
                    opacity: 0.7;
                }

                .prompt-list {
                    display: flex;
                    flex-direction: column;
                    gap: 10px;
                }

                .prompt-item {
                    background: rgba(255, 255, 255, 0.1);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 8px;
                    padding: 15px;
                    cursor: pointer;
                    transition: all 0.3s ease;
                }

                .prompt-item:hover {
                    background: rgba(255, 255, 255, 0.15);
                    border-color: #00ff41;
                    transform: translateX(5px);
                }

                .prompt-item.selected {
                    background: rgba(0, 255, 65, 0.2);
                    border-color: #00ff41;
                    box-shadow: 0 2px 10px rgba(0, 255, 65, 0.3);
                }

                .prompt-text {
                    font-size: 14px;
                    font-weight: 500;
                    margin-bottom: 8px;
                    line-height: 1.4;
                }

                .prompt-meta {
                    display: flex;
                    justify-content: space-between;
                    font-size: 12px;
                    margin-bottom: 8px;
                    opacity: 0.8;
                }

                .confidence {
                    color: #4CAF50;
                    font-weight: 500;
                }

                .tab-title {
                    color: #2196F3;
                    max-width: 150px;
                    overflow: hidden;
                    text-overflow: ellipsis;
                    white-space: nowrap;
                }

                .prompt-context {
                    font-size: 11px;
                    opacity: 0.6;
                    line-height: 1.3;
                    max-height: 40px;
                    overflow: hidden;
                }

                .no-prompts-message {
                    background: rgba(255, 255, 255, 0.1);
                    border-radius: 12px;
                    padding: 30px;
                    text-align: center;
                    border: 1px solid rgba(255, 255, 255, 0.2);
                }

                .scanning-indicator {
                    display: flex;
                    align-items: center;
                    justify-content: center;
                    gap: 10px;
                    margin-bottom: 15px;
                    font-size: 14px;
                }

                .spinner {
                    width: 20px;
                    height: 20px;
                    border: 2px solid rgba(255, 255, 255, 0.3);
                    border-top: 2px solid #00ff41;
                    border-radius: 50%;
                    animation: spin 1s linear infinite;
                }

                @keyframes spin {
                    0% { transform: rotate(0deg); }
                    100% { transform: rotate(360deg); }
                }

                .tabs-info {
                    font-size: 12px;
                    opacity: 0.7;
                }
                "
            </style>
        </div>
    }
}
