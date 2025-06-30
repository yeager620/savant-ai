use leptos::prelude::*;

#[component]
pub fn Dashboard() -> impl IntoView {
    let (provider, set_provider) = signal("ollama".to_string());
    let (model, set_model) = signal("llama3.2".to_string());
    let (temperature, set_temperature) = signal(0.7);
    let (max_tokens, set_max_tokens) = signal(500);
    let (stealth_mode, set_stealth_mode) = signal(true);
    let (auto_scan, set_auto_scan) = signal(false);
    
    view! {
        <div class="dashboard">
            <header class="dashboard-header">
                <h1>"Savant AI - Configuration Dashboard"</h1>
                <p>"Configure your AI assistant settings"</p>
            </header>

            <div class="config-sections">
                <section class="config-section">
                    <h2>"AI Provider Settings"</h2>
                    
                    <div class="form-group">
                        <label for="provider">"Provider:"</label>
                        <select 
                            id="provider" 
                            on:change=move |ev| {
                                set_provider.set(event_target_value(&ev));
                            }
                        >
                            <option value="ollama" selected=move || provider.get() == "ollama">"Ollama (Local)"</option>
                            <option value="openai" selected=move || provider.get() == "openai">"OpenAI"</option>
                            <option value="deepseek" selected=move || provider.get() == "deepseek">"DeepSeek"</option>
                            <option value="anthropic" selected=move || provider.get() == "anthropic">"Anthropic"</option>
                        </select>
                    </div>

                    <div class="form-group">
                        <label for="model">"Model:"</label>
                        <input 
                            type="text" 
                            id="model" 
                            prop:value=move || model.get()
                            on:input=move |ev| {
                                set_model.set(event_target_value(&ev));
                            }
                        />
                    </div>

                    <div class="form-group">
                        <label for="temperature">"Temperature:"</label>
                        <input 
                            type="range" 
                            id="temperature" 
                            min="0" 
                            max="2" 
                            step="0.1"
                            prop:value=move || temperature.get()
                            on:input=move |ev| {
                                let val = event_target_value(&ev).parse::<f64>().unwrap_or(0.7);
                                set_temperature.set(val);
                            }
                        />
                        <span>{move || format!("{:.1}", temperature.get())}</span>
                    </div>

                    <div class="form-group">
                        <label for="max-tokens">"Max Tokens:"</label>
                        <input 
                            type="number" 
                            id="max-tokens" 
                            min="100" 
                            max="4000"
                            prop:value=move || max_tokens.get()
                            on:input=move |ev| {
                                let val = event_target_value(&ev).parse::<i32>().unwrap_or(500);
                                set_max_tokens.set(val);
                            }
                        />
                    </div>

                    <button class="test-btn">"Test Connection"</button>
                </section>

                <section class="config-section">
                    <h2>"Stealth Mode Settings"</h2>
                    <div class="form-group">
                        <label>
                            <input 
                                type="checkbox" 
                                prop:checked=move || stealth_mode.get()
                                on:change=move |ev| {
                                    set_stealth_mode.set(event_target_checked(&ev));
                                }
                            />
                            "Enable Stealth Mode"
                        </label>
                    </div>
                    
                    <div class="form-group">
                        <label>
                            <input 
                                type="checkbox" 
                                prop:checked=move || auto_scan.get()
                                on:change=move |ev| {
                                    set_auto_scan.set(event_target_checked(&ev));
                                }
                            />
                            "Auto-scan for Questions"
                        </label>
                    </div>
                </section>

                <div class="action-buttons">
                    <button class="save-btn">"Save Configuration"</button>
                    <button class="reset-btn">"Reset to Defaults"</button>
                </div>
            </div>
        </div>
    }
}