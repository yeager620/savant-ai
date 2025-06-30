use leptos::prelude::*;
use crate::components::MinimalChat;

#[component]
pub fn TaskbarApp() -> impl IntoView {
    view! {
        <div class="taskbar-app">
            <MinimalChat />
            
            // Taskbar-specific CSS
            <style>
                "
                * {
                    margin: 0;
                    padding: 0;
                    box-sizing: border-box;
                    font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
                }

                body {
                    background: transparent;
                    color: #ffffff;
                    font-size: 13px;
                    overflow: hidden;
                }

                .taskbar-app {
                    width: 100vw;
                    height: 100vh;
                    display: flex;
                    flex-direction: column;
                    background: rgba(16, 16, 16, 0.75);
                    backdrop-filter: blur(20px) saturate(180%);
                    border-right: 1px solid rgba(255, 255, 255, 0.1);
                }

                .minimal-chat {
                    display: flex;
                    flex-direction: column;
                    height: 100%;
                    padding: 12px;
                }

                .chat-header {
                    display: flex;
                    justify-content: space-between;
                    align-items: center;
                    padding: 8px 0;
                    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
                    margin-bottom: 12px;
                }
                
                .header-right {
                    display: flex;
                    align-items: center;
                    gap: 12px;
                }
                
                .context-usage {
                    display: flex;
                    flex-direction: column;
                    align-items: flex-end;
                    gap: 2px;
                }
                
                .context-text {
                    font-size: 10px;
                    color: rgba(255, 255, 255, 0.7);
                    font-weight: 500;
                }
                
                .context-breakdown {
                    font-size: 8px;
                    color: rgba(255, 255, 255, 0.5);
                    font-weight: 400;
                    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
                }
                
                .context-bar {
                    width: 60px;
                    height: 3px;
                    background: rgba(255, 255, 255, 0.2);
                    border-radius: 2px;
                    overflow: hidden;
                }
                
                .context-fill {
                    height: 100%;
                    background: linear-gradient(90deg, #10b981 0%, #f59e0b 70%, #ef4444 100%);
                    border-radius: 2px;
                    transition: width 0.3s ease;
                }
                
                .context-warning {
                    display: flex;
                    align-items: center;
                    gap: 6px;
                    padding: 8px 12px;
                    margin-bottom: 8px;
                    background: rgba(245, 158, 11, 0.1);
                    border: 1px solid rgba(245, 158, 11, 0.3);
                    border-radius: 6px;
                    animation: slide-in 0.3s ease-out;
                }
                
                .warning-icon {
                    color: #f59e0b;
                    font-size: 14px;
                    font-weight: bold;
                }
                
                .warning-text {
                    color: rgba(255, 255, 255, 0.9);
                    font-size: 11px;
                    line-height: 1.3;
                }
                
                @keyframes slide-in {
                    from {
                        opacity: 0;
                        transform: translateY(-10px);
                    }
                    to {
                        opacity: 1;
                        transform: translateY(0);
                    }
                }

                .chat-header h3 {
                    font-size: 14px;
                    font-weight: 600;
                    color: #ffffff;
                }

                .status-indicator {
                    font-size: 8px;
                    color: #10b981;
                    animation: pulse 2s infinite;
                }

                .status-indicator.streaming {
                    color: #00ff41;
                    animation: streaming-pulse 0.8s infinite;
                }

                @keyframes pulse {
                    0%, 100% { opacity: 1; }
                    50% { opacity: 0.5; }
                }

                @keyframes streaming-pulse {
                    0%, 100% { opacity: 0.6; }
                    50% { opacity: 1; }
                }

                .chat-messages {
                    flex: 1;
                    overflow-y: auto;
                    padding: 4px 0;
                    margin-bottom: 12px;
                    display: flex;
                    flex-direction: column;
                    gap: 8px;
                }

                .chat-messages::-webkit-scrollbar {
                    width: 4px;
                }

                .chat-messages::-webkit-scrollbar-track {
                    background: rgba(255, 255, 255, 0.05);
                    border-radius: 2px;
                }

                .chat-messages::-webkit-scrollbar-thumb {
                    background: rgba(255, 255, 255, 0.2);
                    border-radius: 2px;
                }

                .message {
                    padding: 8px 10px;
                    border-radius: 8px;
                    max-width: 100%;
                    word-wrap: break-word;
                    position: relative;
                }

                .message.user {
                    background: rgba(59, 130, 246, 0.15);
                    border: 1px solid rgba(59, 130, 246, 0.3);
                    align-self: flex-end;
                    margin-left: 20px;
                }

                .message.ai {
                    background: rgba(16, 185, 129, 0.15);
                    border: 1px solid rgba(16, 185, 129, 0.3);
                    align-self: flex-start;
                    margin-right: 20px;
                }

                .message.typing {
                    background: rgba(107, 114, 128, 0.15);
                    border: 1px solid rgba(107, 114, 128, 0.3);
                }
                
                .message.streaming {
                    background: rgba(16, 185, 129, 0.15);
                    border: 1px solid rgba(16, 185, 129, 0.3);
                    align-self: flex-start;
                    margin-right: 20px;
                }

                .message-content {
                    font-size: 12px;
                    line-height: 1.4;
                    color: #ffffff;
                }
                
                .message-content p {
                    margin: 0 0 8px 0;
                }
                
                .message-content p:last-child {
                    margin-bottom: 0;
                }
                
                .message-content code {
                    background: rgba(255, 255, 255, 0.1);
                    padding: 2px 4px;
                    border-radius: 3px;
                    font-family: 'SF Mono', Monaco, 'Cascadia Code', 'Roboto Mono', Consolas, 'Courier New', monospace;
                    font-size: 11px;
                }
                
                .message-content pre {
                    background: rgba(255, 255, 255, 0.05);
                    padding: 8px;
                    border-radius: 4px;
                    overflow-x: auto;
                    margin: 4px 0;
                }
                
                .message-content pre code {
                    background: none;
                    padding: 0;
                }

                .message-time {
                    font-size: 10px;
                    color: rgba(255, 255, 255, 0.5);
                    margin-top: 4px;
                    text-align: right;
                }

                .typing-indicator {
                    display: flex;
                    gap: 2px;
                    align-items: center;
                }

                .typing-indicator span {
                    width: 4px;
                    height: 4px;
                    background: rgba(255, 255, 255, 0.6);
                    border-radius: 50%;
                    animation: typing 1.4s infinite ease-in-out;
                }

                .typing-indicator span:nth-child(1) { animation-delay: 0s; }
                .typing-indicator span:nth-child(2) { animation-delay: 0.2s; }
                .typing-indicator span:nth-child(3) { animation-delay: 0.4s; }

                @keyframes typing {
                    0%, 60%, 100% {
                        transform: translateY(0);
                        opacity: 0.4;
                    }
                    30% {
                        transform: translateY(-10px);
                        opacity: 1;
                    }
                }

                .chat-input {
                    display: flex;
                    gap: 8px;
                    align-items: flex-end;
                }

                .chat-input textarea {
                    flex: 1;
                    background: rgba(255, 255, 255, 0.05);
                    border: 1px solid rgba(255, 255, 255, 0.2);
                    border-radius: 6px;
                    padding: 8px 10px;
                    color: #ffffff;
                    font-size: 12px;
                    resize: none;
                    min-height: 36px;
                    max-height: 100px;
                    font-family: inherit;
                }

                .chat-input textarea::placeholder {
                    color: rgba(255, 255, 255, 0.4);
                }

                .chat-input textarea:focus {
                    outline: none;
                    border-color: rgba(59, 130, 246, 0.6);
                    box-shadow: 0 0 0 2px rgba(59, 130, 246, 0.2);
                }

                .chat-input textarea:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                }

                .chat-input button {
                    background: rgba(59, 130, 246, 0.8);
                    border: none;
                    border-radius: 6px;
                    padding: 8px 12px;
                    color: #ffffff;
                    font-size: 11px;
                    font-weight: 500;
                    cursor: pointer;
                    transition: all 0.2s ease;
                    min-width: 50px;
                    height: 36px;
                }

                .chat-input button:hover:not(:disabled) {
                    background: rgba(59, 130, 246, 1);
                    transform: translateY(-1px);
                }

                .chat-input button:disabled {
                    opacity: 0.5;
                    cursor: not-allowed;
                    transform: none;
                }

                .chat-input button:active {
                    transform: translateY(0);
                }
                "
            </style>
        </div>
    }
}