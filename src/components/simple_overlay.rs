use leptos::prelude::*;

#[component]
pub fn SimpleOverlay() -> impl IntoView {
    view! {
        <div id="ai-overlay" class="overlay-container">
            <div class="processing-indicator">
                <span>"AI Overlay (Coming Soon)"</span>
            </div>
        </div>
    }
}

#[component]
pub fn SimpleOverlayToggle() -> impl IntoView {
    let (overlay_active, set_overlay_active) = signal(false);
    
    let toggle_overlay = move |_| {
        set_overlay_active.update(|active| *active = !*active);
    };
    
    view! {
        <button 
            class="overlay-toggle"
            class:active=move || overlay_active.get()
            on:click=toggle_overlay
        >
            {move || if overlay_active.get() { "🔍 Stop Scanning" } else { "🔍 Start Scanning" }}
        </button>
    }
}