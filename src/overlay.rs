use leptos::prelude::*;
use crate::components::MinimalOverlay;

#[component]
pub fn OverlayApp() -> impl IntoView {
    view! {
        <div class="overlay-root">
            <MinimalOverlay />
            
            // Neon Green CSS Styles
            <style>
                "
                * {
                    margin: 0;
                    padding: 0;
                    box-sizing: border-box;
                }

                .overlay-root {
                    width: 100vw;
                    height: 100vh;
                    position: fixed;
                    top: 0;
                    left: 0;
                    pointer-events: none;
                    z-index: 9999;
                }

                .minimal-overlay {
                    width: 100%;
                    height: 100%;
                    position: relative;
                }

                .scan-indicator {
                    position: fixed;
                    top: 10px;
                    right: 10px;
                    width: 10px;
                    height: 10px;
                    background: #00FF41;
                    border-radius: 50%;
                    opacity: 0;
                    transition: opacity 0.3s;
                }

                .scan-indicator.active {
                    opacity: 0.7;
                    animation: scan-pulse 2s infinite;
                }

                @keyframes scan-pulse {
                    0%, 100% { opacity: 0.3; }
                    50% { opacity: 0.9; }
                }

                .demo-neon-box {
                    position: absolute;
                    top: 100px;
                    left: 100px;
                    width: 300px;
                    height: 30px;
                    border: 2px solid #00FF41;
                    border-radius: 8px;
                    background: rgba(0, 255, 65, 0.1);
                    box-shadow: 
                        0 0 10px #00FF41,
                        inset 0 0 10px rgba(0, 255, 65, 0.1);
                    animation: neon-pulse 2s infinite;
                }

                .demo-response {
                    position: absolute;
                    top: 40px;
                    left: 0;
                    color: #00FF41;
                    font-family: 'Courier New', monospace;
                    font-size: 14px;
                    background: rgba(0, 0, 0, 0.8);
                    padding: 12px 16px;
                    border-radius: 8px;
                    border: 1px solid #00FF41;
                    box-shadow: 
                        0 0 15px rgba(0, 255, 65, 0.3),
                        inset 0 0 10px rgba(0, 255, 65, 0.1);
                    text-shadow: 0 0 8px #00FF41;
                    white-space: nowrap;
                }

                .neon-question-container {
                    pointer-events: none;
                    opacity: 1;
                    transition: opacity 1s ease-out;
                }

                .neon-question-container.fade-out {
                    opacity: 0;
                }

                .neon-question-box {
                    border: 2px solid #00FF41;
                    border-radius: 8px;
                    background: rgba(0, 255, 65, 0.1);
                    box-shadow: 
                        0 0 10px #00FF41,
                        inset 0 0 10px rgba(0, 255, 65, 0.1);
                    animation: neon-pulse 2s infinite;
                    pointer-events: none;
                }

                @keyframes neon-pulse {
                    0%, 100% {
                        box-shadow: 
                            0 0 10px #00FF41,
                            0 0 20px #00FF41,
                            inset 0 0 10px rgba(0, 255, 65, 0.1);
                    }
                    50% {
                        box-shadow: 
                            0 0 15px #00FF41,
                            0 0 30px #00FF41,
                            inset 0 0 15px rgba(0, 255, 65, 0.2);
                    }
                }

                .neon-response-text {
                    color: #00FF41;
                    font-family: 'Courier New', monospace;
                    font-size: 14px;
                    font-weight: 500;
                    background: rgba(0, 0, 0, 0.8);
                    padding: 12px 16px;
                    border-radius: 8px;
                    border: 1px solid #00FF41;
                    box-shadow: 
                        0 0 15px rgba(0, 255, 65, 0.3),
                        inset 0 0 10px rgba(0, 255, 65, 0.1);
                    backdrop-filter: blur(10px);
                    line-height: 1.4;
                    max-width: 400px;
                    word-wrap: break-word;
                    pointer-events: none;
                    text-shadow: 0 0 8px #00FF41;
                    animation: typewriter 0.1s steps(1, end);
                }

                @keyframes typewriter {
                    from {
                        width: 0;
                    }
                    to {
                        width: 100%;
                    }
                }

                .neon-response-text::before {
                    content: '';
                    position: absolute;
                    top: -2px;
                    left: -2px;
                    right: -2px;
                    bottom: -2px;
                    background: linear-gradient(45deg, transparent, #00FF41, transparent);
                    border-radius: 8px;
                    z-index: -1;
                    animation: border-glow 3s linear infinite;
                }

                @keyframes border-glow {
                    0%, 100% {
                        opacity: 0.5;
                    }
                    50% {
                        opacity: 1;
                    }
                }

                /* Ensure overlay is completely invisible when no questions detected */
                .invisible-overlay-container:empty {
                    display: none;
                }

                /* Responsive text sizing */
                @media (max-width: 768px) {
                    .neon-response-text {
                        font-size: 12px;
                        max-width: 300px;
                        padding: 10px 14px;
                    }
                }

                @media (max-width: 480px) {
                    .neon-response-text {
                        font-size: 11px;
                        max-width: 250px;
                        padding: 8px 12px;
                    }
                }
                "
            </style>
        </div>
    }
}