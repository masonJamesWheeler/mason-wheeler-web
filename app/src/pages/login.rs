use leptos::prelude::*;

#[component]
pub fn LoginPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (loading, set_loading) = signal(false);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        #[cfg(feature = "hydrate")]
        {
            let email_val = email.get();
            let password_val = password.get();

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
                    <div class="w-12 h-12 rounded-xl bg-stone-900 flex items-center justify-center mx-auto mb-5 shadow-lg shadow-stone-900/10">
                        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                        </svg>
                    </div>
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
                            class="input"
                            placeholder="you@example.com"
                            on:input=move |ev| set_email.set(event_target_value(&ev))
                            prop:value=move || email.get()
                        />
                    </div>
                    <div>
                        <label class="block text-xs font-medium text-stone-500 mb-1.5 ml-1" for="password">"Password"</label>
                        <input
                            id="password"
                            type="password"
                            required=true
                            autocomplete="current-password"
                            class="input"
                            placeholder="Enter your password"
                            on:input=move |ev| set_password.set(event_target_value(&ev))
                            prop:value=move || password.get()
                        />
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
