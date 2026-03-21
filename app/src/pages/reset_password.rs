#![allow(unused)]
use leptos::prelude::*;

#[component]
pub fn ResetPasswordPage() -> impl IntoView {
    let (new_password, set_new_password) = signal(String::new());
    let (confirm_password, set_confirm_password) = signal(String::new());
    let (error, set_error) = signal::<Option<String>>(None);
    let (success, set_success) = signal(false);
    let (loading, set_loading) = signal(false);

    // Extract token from URL query params
    let (token, _) = signal({
        #[cfg(feature = "hydrate")]
        {
            web_sys::window()
                .and_then(|w| w.location().search().ok())
                .and_then(|s| {
                    web_sys::UrlSearchParams::new_with_str(&s)
                        .ok()
                        .and_then(|params| params.get("token"))
                })
                .unwrap_or_default()
        }
        #[cfg(not(feature = "hydrate"))]
        {
            String::new()
        }
    });

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        let pw = new_password.get();
        let confirm = confirm_password.get();

        if pw != confirm {
            set_error.set(Some("Passwords do not match.".to_string()));
            return;
        }

        if pw.len() < 8 {
            set_error.set(Some("Password must be at least 8 characters.".to_string()));
            return;
        }

        set_loading.set(true);

        #[cfg(feature = "hydrate")]
        {
            let token_val = token.get();
            let password_val = pw;

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "token": token_val,
                    "new_password": password_val,
                });

                match gloo_net::http::Request::post("/api/auth/reset-password")
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    Ok(resp) => {
                        if resp.ok() {
                            set_success.set(true);
                            // Redirect to login after 2 seconds
                            leptos::task::spawn_local(async move {
                                gloo_timers::future::TimeoutFuture::new(2000).await;
                                if let Some(window) = web_sys::window() {
                                    let _ = window.location().set_href("/login");
                                }
                            });
                        } else {
                            set_error.set(Some(
                                "Invalid or expired reset link. Please request a new one.".to_string(),
                            ));
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
                    <h1 class="text-xl font-semibold text-stone-900 tracking-tight">"Reset password"</h1>
                    <p class="text-stone-400 text-sm mt-1">"Enter your new password"</p>
                </div>

                // Success message
                <Show when=move || success.get()>
                    <div class="mb-6 px-4 py-3 rounded-xl bg-green-50 border border-green-100 flex items-start gap-3">
                        <svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <p class="text-sm text-green-700">"Password reset! Redirecting to login..."</p>
                    </div>
                </Show>

                // Error
                <Show when=move || error.get().is_some()>
                    <div class="mb-6 px-4 py-3 rounded-xl bg-red-50 border border-red-100 flex items-start gap-3">
                        <svg class="w-5 h-5 text-red-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                        </svg>
                        <p class="text-sm text-red-700">{move || error.get().unwrap_or_default()}</p>
                    </div>
                </Show>

                // Form (hidden after success)
                <Show when=move || !success.get()>
                    <form on:submit=on_submit class="space-y-4">
                        <div>
                            <label class="block text-xs font-medium text-stone-500 mb-1.5 ml-1" for="new-password">"New password"</label>
                            <input
                                id="new-password"
                                type="password"
                                required=true
                                autocomplete="new-password"
                                class="input"
                                placeholder="Enter new password"
                                on:input=move |ev| set_new_password.set(event_target_value(&ev))
                                prop:value=move || new_password.get()
                            />
                        </div>
                        <div>
                            <label class="block text-xs font-medium text-stone-500 mb-1.5 ml-1" for="confirm-password">"Confirm password"</label>
                            <input
                                id="confirm-password"
                                type="password"
                                required=true
                                autocomplete="new-password"
                                class="input"
                                placeholder="Confirm new password"
                                on:input=move |ev| set_confirm_password.set(event_target_value(&ev))
                                prop:value=move || confirm_password.get()
                            />
                        </div>
                        <button
                            type="submit"
                            class="btn-primary w-full py-3 text-[15px] mt-2"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() {
                                "Resetting..."
                            } else {
                                "Reset password"
                            }}
                        </button>
                    </form>
                </Show>
            </div>
        </div>
    }
}
