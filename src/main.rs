mod components;
mod app;
mod utils;
mod overlay;

use app::*;
use overlay::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    
    // Determine which component to mount based on URL path
    let path = web_sys::window()
        .unwrap()
        .location()
        .pathname()
        .unwrap_or_default();
    
    if path.contains("overlay") || path == "/overlay" {
        // Mount invisible overlay for fullscreen transparent window
        mount_to_body(|| {
            view! {
                <OverlayApp/>
            }
        })
    } else {
        // Mount main dashboard app
        mount_to_body(|| {
            view! {
                <App/>
            }
        })
    }
}
