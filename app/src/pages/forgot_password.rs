#![allow(unused)]
use leptos::prelude::*;

#[component]
pub fn ForgotPasswordPage() -> impl IntoView {
    let (email, set_email) = signal(String::new());
    let (submitted, set_submitted) = signal(false);
    let (loading, set_loading) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let on_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_loading.set(true);
        set_error.set(None);

        #[cfg(feature = "hydrate")]
        {
            let email_val = email.get();

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "email": email_val,
                });

                match crate::api::client::api_post_no_body("/api/auth/forgot-password", &body).await {
                    Ok(()) => {
                        set_submitted.set(true);
                    }
                    Err(e) => {
                        set_error.set(Some(e.message));
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
                    <h1 class="text-xl font-semibold text-stone-900 tracking-tight">"Forgot password"</h1>
                    <p class="text-stone-400 text-sm mt-1">"Enter your email to receive a reset link"</p>
                </div>

                // Success message
                <Show when=move || submitted.get()>
                    <div class="mb-6 px-4 py-3 rounded-xl bg-green-50 border border-green-100 flex items-start gap-3">
                        <svg class="w-5 h-5 text-green-500 mt-0.5 flex-shrink-0" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                        <p class="text-sm text-green-700">"If an account exists with that email, a reset link has been sent."</p>
                    </div>
                    <p class="text-center text-sm text-stone-500 mt-4">
                        <a href="/login" class="text-stone-700 hover:text-stone-900 underline underline-offset-2 decoration-stone-300 transition-colors">
                            "Back to login"
                        </a>
                    </p>
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

                // Form (hidden after submission)
                <Show when=move || !submitted.get()>
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
                        <button
                            type="submit"
                            class="btn-primary w-full py-3 text-[15px] mt-2"
                            disabled=move || loading.get()
                        >
                            {move || if loading.get() {
                                "Sending..."
                            } else {
                                "Send reset link"
                            }}
                        </button>
                    </form>

                    <p class="text-center text-sm text-stone-400 mt-6">
                        <a href="/login" class="text-stone-500 hover:text-stone-700 underline underline-offset-2 decoration-stone-300 transition-colors">
                            "Back to login"
                        </a>
                    </p>
                </Show>
            </div>
        </div>
    }
}
