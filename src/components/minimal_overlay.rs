use leptos::prelude::*;

#[component]
pub fn MinimalOverlay() -> impl IntoView {
    view! {
        <div class="minimal-overlay">
            <div class="demo-neon-box">
                "Question detected: What is the capital of France?"
            </div>
            <div class="demo-response">
                "Paris is the capital and largest city of France."
            </div>
        </div>
    }
}