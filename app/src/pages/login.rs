use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (email_error, set_email_error) = signal::<Option<String>>(None);
    let (password_error, set_password_error) = signal::<Option<String>>(None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);
        set_email_error.set(None);
        set_password_error.set(None);

        // Client-side validation
        let email_val = email.get();
        let password_val = password.get();
        let mut has_error = false;

        if email_val.trim().is_empty() {
            set_email_error.set(Some("Email is required".to_string()));
            has_error = true;
        } else if !email_val.contains('@') {
            set_email_error.set(Some("Please enter a valid email address".to_string()));
            has_error = true;
        }

        if password_val.is_empty() {
            set_password_error.set(Some("Password is required".to_string()));
            has_error = true;
        }

        if has_error {
            return;
        }

        set_loading.set(true);

        #[cfg(feature = "hydrate")]
        {

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "email": email_val,
                    "password": password_val,
                });

                match gloo_net::http::Request::post("/api/auth/login")
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.ok() {
                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                let role = data
                                    .get("user")
                                    .and_then(|u| u.get("role"))
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("tenant");

                                let redirect = if role == "landlord" { "/admin" } else { "/tenant" };

                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href(redirect);
                                }
                            }
                        } else {
                            let msg = resp
                                .json::<serde_json::Value>()
                                .await
                                .ok()
                                .and_then(|v| v.get("error").and_then(|e| e.as_str().map(String::from)))
                                .unwrap_or_else(|| "Invalid credentials".to_string());
                            set_error.set(Some(msg));
                        }
                    }
                    Err(_) => {
                        set_error.set(Some("Unable to connect. Please try again.".to_string()));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="min-h-[85vh] flex items-center justify-center px-6">
            <div class="w-full max-w-[380px]">
                // Logo
                <div class="text-center mb-10">
                    <p class="text-xs font-semibold uppercase tracking-[0.1em] text-stone-400 mb-3">"8404 12th Ave S"</p>
                    <h1 class="text-xl font-semibold text-stone-900 tracking-tight">"Welcome back"</h1>
                    <p class="text-stone-400 text-sm mt-1">"Sign in to your resident portal"</p>
                </div>

                // Error
                <Show when=move || error.get().is_some()>
                    <div class="mb-6 px-4 py-3 rounded-xl bg-red-50 border border-red-100 flex items-start gap-3">
                        <svg class="w-5 h-5 text-red-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                        </svg>
                        <p class="text-sm text-red-700">{move || error.get().unwrap_or_default()}</p>
                    </div>
                </Show>

                // Form
                <form on:submit=on_submit class="space-y-4">
                    <div>
                        <label class="block text-xs font-medium text-stone-500 mb-1.5 ml-1" for="email">"Email"</label>
                        <input
                            id="email"
                            type="email"
                            required=true
                            autocomplete="email"
                            class=move || if email_error.get().is_some() { "input border-red-300" } else { "input" }
                            placeholder="you@example.com"
                            on:input=move |ev| {
                                set_email.set(event_target_value(&ev));
                                set_email_error.set(None);
                            }
                            prop:value=move || email.get()
                        />
                        <Show when=move || email_error.get().is_some()>
                            <p class="text-xs text-red-500 mt-1 ml-1">{move || email_error.get().unwrap_or_default()}</p>
                        </Show>
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-stone-500 mb-1.5 ml-1" for="password">"Password"</label>
                        <input
                            id="password"
                            type="password"
                            required=true
                            autocomplete="current-password"
                            class=move || if password_error.get().is_some() { "input border-red-300" } else { "input" }
                            placeholder="Enter your password"
                            on:input=move |ev| {
                                set_password.set(event_target_value(&ev));
                                set_password_error.set(None);
                            }
                            prop:value=move || password.get()
                        />
                        <Show when=move || password_error.get().is_some()>
                            <p class="text-xs text-red-500 mt-1 ml-1">{move || password_error.get().unwrap_or_default()}</p>
                        </Show>
                    </div>
                    <button
                        type="submit"
                        class="btn-primary w-full py-3 text-[15px] mt-2"
                        disabled=move || loading.get()
                    >
                        {move || if loading.get() {
                            "Signing in..."
                        } else {
                            "Sign in"
                        }}
                    </button>
                </form>

                <p class="text-center text-sm text-stone-400 mt-4">
                    <a href="/forgot-password" class="text-stone-500 hover:text-stone-700 underline underline-offset-2 decoration-stone-300 transition-colors">
                        "Forgot password?"
                    </a>
                </p>

                <p class="text-center text-xs text-stone-400 mt-8">
                    "Need access? Contact "
                    <a href="mailto:masonwheeler@fieldflow.us" class="text-stone-500 hover:text-stone-700 underline underline-offset-2 decoration-stone-300 transition-colors">
                        "your landlord"
                    </a>
                </p>
            </div>
        </div>
    }
}
