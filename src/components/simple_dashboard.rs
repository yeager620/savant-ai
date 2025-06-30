use leptos::prelude::*;

#[component]
pub fn SimpleDashboard() -> impl IntoView {
    view! {
        <div class="dashboard">
            <h1>"Savant AI Configuration"</h1>
            <div class="config-section">
                <h2>"AI Configuration"</h2>
                <p>"Configuration UI will be implemented soon..."</p>
            </div>
            <div class="config-section">
                <h2>"Application Settings"</h2>
                <p>"Settings UI will be implemented soon..."</p>
            </div>
        </div>
    }
}