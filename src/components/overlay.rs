use leptos::prelude::*;

#[component]
pub fn Overlay() -> impl IntoView {
    let (overlay_active, set_overlay_active) = signal(false);
    
    view! {
        <div class="overlay-container">
            <div class="overlay-status">
                <p>"AI Overlay System"</p>
                <p class="status-text">
                    {move || if overlay_active.get() { 
                        "🔍 Scanning for questions..." 
                    } else { 
                        "⏸️ Overlay paused" 
                    }}
                </p>
                
                <button 
                    class="overlay-toggle"
                    on:click=move |_| {
                        set_overlay_active.set(!overlay_active.get());
                    }
                >
                    {move || if overlay_active.get() { "Stop Scanning" } else { "Start Scanning" }}
                </button>
            </div>
            
            <div class="demo-bubble" class:hidden=move || !overlay_active.get()>
                <div class="bubble-header">
                    <span class="question-preview">"Example question detected"</span>
                    <button class="close-btn">"×"</button>
                </div>
                <div class="bubble-content">
                    <p>"This is where AI answers would appear when questions are detected on screen."</p>
                    <small>"Powered by Savant AI"</small>
                </div>
            </div>
        </div>
    }
}