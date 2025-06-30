use leptos::prelude::*;

#[component]
pub fn SimpleApp() -> impl IntoView {
    let (count, set_count) = signal(0);

    view! {
        <div class="app">
            <h1>"Savant AI - Development Preview"</h1>
            <p>"Count: " {count}</p>
            <button on:click=move |_| set_count.update(|n| *n += 1)>
                "Click me!"
            </button>
            <div class="info">
                <p>"This is a development preview of the Savant AI assistant."</p>
                <p>"Full features will be implemented soon."</p>
            </div>
        </div>
    }
}