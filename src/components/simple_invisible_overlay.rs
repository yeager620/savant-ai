use leptos::prelude::*;
use leptos::task::spawn_local;

#[component]
pub fn SimpleInvisibleOverlay() -> impl IntoView {
    let (scanning_active, set_scanning_active) = signal(false);

    // Auto-start scanning when component mounts
    Effect::new(move |_| {
        spawn_local(async move {
            // Start the background scanning
            set_scanning_active.set(true);
        });
    });

    view! {
        <div class="simple-invisible-overlay">
            // This will be completely transparent and cover the full screen
            <div class="scan-indicator" class:active=scanning_active>
                // Hidden indicator that scanning is active
            </div>
            
            // Demo neon box for testing (will be replaced by dynamic detection)
            <div class="demo-neon-box">
                <div class="demo-response">
                    "Demo: AI responses will appear here when questions are detected"
                </div>
            </div>
        </div>
    }
}