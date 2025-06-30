mod components;
mod utils;
mod taskbar_app;
mod app;

use taskbar_app::*;
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    
    // Mount the minimal taskbar app
    mount_to_body(|| {
        view! {
            <TaskbarApp/>
        }
    })
}
