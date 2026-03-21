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
                            // Parse response to get role
                            if let Ok(data) = resp.json::<serde_json::Value>().await {
                                let role = data
                                    .get("user")
                                    .and_then(|u| u.get("role"))
                                    .and_then(|r| r.as_str())
                                    .unwrap_or("tenant");

                                let redirect = if role == "landlord" {
                                    "/admin"
                                } else {
                                    "/tenant"
                                };

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
                                .unwrap_or_else(|| "Login failed".to_string());
                            set_error.set(Some(msg));
                        }
                    }
                    Err(e) => {
                        set_error.set(Some(format!("Network error: {}", e)));
                    }
                }
                set_loading.set(false);
            });
        }
    };

    view! {
        <div class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="w-full max-w-md">
                <div class="bg-white border border-slate-200 rounded-xl shadow-sm p-8">
                    <h1 class="text-2xl font-bold text-slate-900 text-center mb-2">"Tenant Portal"</h1>
                    <p class="text-slate-500 text-center mb-8">"Sign in to manage your lease"</p>

                    <Show when=move || error.get().is_some()>
                        <div class="bg-red-50 border border-red-200 text-red-700 px-4 py-3 rounded-lg mb-6 text-sm">
                            {move || error.get().unwrap_or_default()}
                        </div>
                    </Show>

                    <form on:submit=on_submit>
                        <div class="space-y-4">
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="email">
                                    "Email"
                                </label>
                                <input
                                    id="email"
                                    type="email"
                                    required=true
                                    class="w-full px-4 py-3 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors"
                                    placeholder="you@example.com"
                                    on:input=move |ev| set_email.set(event_target_value(&ev))
                                    prop:value=move || email.get()
                                />
                            </div>
                            <div>
                                <label class="block text-sm font-medium text-slate-700 mb-1" for="password">
                                    "Password"
                                </label>
                                <input
                                    id="password"
                                    type="password"
                                    required=true
                                    class="w-full px-4 py-3 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none transition-colors"
                                    placeholder="Enter your password"
                                    on:input=move |ev| set_password.set(event_target_value(&ev))
                                    prop:value=move || password.get()
                                />
                            </div>
                            <button
                                type="submit"
                                class="w-full bg-orange-600 hover:bg-orange-700 text-white font-medium py-3 rounded-lg transition-colors disabled:opacity-50"
                                disabled=move || loading.get()
                            >
                                {move || if loading.get() { "Signing in..." } else { "Sign In" }}
                            </button>
                        </div>
                    </form>
                </div>
                <p class="text-center text-sm text-slate-500 mt-6">
                    "Need access? Contact your landlord at "
                    <a href="mailto:masonwheeler@fieldflow.us" class="text-orange-600 hover:underline">
                        "masonwheeler@fieldflow.us"
                    </a>
                </p>
            </div>
        </div>
    }
}
