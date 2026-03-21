#![allow(unused)]
use leptos::prelude::*;

#[component]
pub fn ApplyPage() -> impl IntoView {
    let (name, set_name) = signal(String::new());
    let (email, set_email) = signal(String::new());
    let (phone, set_phone) = signal(String::new());
    let (move_in, set_move_in) = signal(String::new());
    let (message, set_message) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (success, set_success) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_error.set(None);

        // Client-side validation
        if name.get().trim().is_empty() {
            set_error.set(Some("Name is required.".to_string()));
            return;
        }
        let email_val = email.get();
        if email_val.trim().is_empty() || !email_val.contains('@') {
            set_error.set(Some("A valid email address is required.".to_string()));
            return;
        }

        set_submitting.set(true);

        #[cfg(feature = "hydrate")]
        {
            let name_val = name.get();
            let email_val = email.get();
            let phone_val = phone.get();
            let move_in_val = move_in.get();
            let message_val = message.get();

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "name": name_val,
                    "email": email_val,
                    "phone": if phone_val.is_empty() { None } else { Some(phone_val) },
                    "desired_move_in": if move_in_val.is_empty() { None } else { Some(move_in_val) },
                    "message": if message_val.is_empty() { None } else { Some(message_val) },
                });

                match crate::api::client::api_post_no_body("/api/apply", &body).await {
                    Ok(()) => {
                        set_success.set(true);
                    }
                    Err(e) => {
                        set_error.set(Some(e.message));
                    }
                }
                set_submitting.set(false);
            });
        }
    };

    let input_class = "w-full rounded-lg border border-stone-300 px-4 py-2.5 text-sm text-stone-900 placeholder:text-stone-400 focus:border-stone-500 focus:outline-none focus:ring-1 focus:ring-stone-500 transition-colors";
    let label_class = "block text-sm font-medium text-stone-700 mb-1.5";

    view! {
        <div class="min-h-[calc(100vh-3.5rem)] bg-stone-50">
            <div class="mx-auto max-w-3xl px-6 py-12">
                // Property header
                <div class="mb-10 text-center">
                    <h1 class="text-2xl sm:text-3xl font-semibold text-stone-900 tracking-tight">"Apply to Rent"</h1>
                    <div class="mt-3 inline-flex items-center gap-2 rounded-full bg-stone-100 px-4 py-2 text-sm text-stone-600">
                        <svg class="h-4 w-4 text-stone-400" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                        </svg>
                        "8404 12th Ave S, Seattle WA  —  2bd/1ba, $3,250/mo"
                    </div>
                </div>

                <Show when=move || success.get()>
                    <div class="rounded-xl border border-green-200 bg-green-50 p-8 text-center">
                        <div class="mx-auto mb-4 flex h-12 w-12 items-center justify-center rounded-full bg-green-100">
                            <svg class="h-6 w-6 text-green-600" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M4.5 12.75l6 6 9-13.5" />
                            </svg>
                        </div>
                        <h2 class="text-lg font-semibold text-green-900">"Application Received!"</h2>
                        <p class="mt-2 text-sm text-green-700">
                            "We'll review your information and contact you within 24-48 hours."
                        </p>
                        <a href="/" class="mt-6 inline-flex items-center gap-1.5 text-sm font-medium text-green-700 hover:text-green-900 transition-colors">
                            <svg class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M10.5 19.5L3 12m0 0l7.5-7.5M3 12h18" />
                            </svg>
                            "Back to home"
                        </a>
                    </div>
                </Show>

                <Show when=move || !success.get()>
                    <div class="rounded-xl border border-stone-200 bg-white p-6 sm:p-8 shadow-sm">
                        // Error message
                        <Show when=move || error.get().is_some()>
                            <div class="mb-6 rounded-lg border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
                                {move || error.get().unwrap_or_default()}
                            </div>
                        </Show>

                        <form on:submit=handle_submit>
                            <div class="space-y-5">
                                // Name
                                <div>
                                    <label class=label_class>"Full Name " <span class="text-red-500">"*"</span></label>
                                    <input
                                        type="text"
                                        class=input_class
                                        placeholder="Jane Smith"
                                        prop:value=move || name.get()
                                        on:input=move |ev| set_name.set(event_target_value(&ev))
                                        required
                                    />
                                </div>

                                // Email
                                <div>
                                    <label class=label_class>"Email " <span class="text-red-500">"*"</span></label>
                                    <input
                                        type="email"
                                        class=input_class
                                        placeholder="jane@example.com"
                                        prop:value=move || email.get()
                                        on:input=move |ev| set_email.set(event_target_value(&ev))
                                        required
                                    />
                                </div>

                                // Phone
                                <div>
                                    <label class=label_class>"Phone"</label>
                                    <input
                                        type="tel"
                                        class=input_class
                                        placeholder="(206) 555-0123"
                                        prop:value=move || phone.get()
                                        on:input=move |ev| set_phone.set(event_target_value(&ev))
                                    />
                                </div>

                                // Desired move-in date
                                <div>
                                    <label class=label_class>"Desired Move-in Date"</label>
                                    <input
                                        type="date"
                                        class=input_class
                                        prop:value=move || move_in.get()
                                        on:input=move |ev| set_move_in.set(event_target_value(&ev))
                                    />
                                </div>

                                // Optional message (collapsed by default)
                                {
                                    let (show_message, set_show_message) = signal(false);
                                    view! {
                                        <div>
                                            <Show when=move || !show_message.get()>
                                                <button
                                                    type="button"
                                                    class="text-sm text-stone-500 hover:text-stone-700 transition-colors flex items-center gap-1"
                                                    on:click=move |_| set_show_message.set(true)
                                                >
                                                    <svg class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M12 4.5v15m7.5-7.5h-15" />
                                                    </svg>
                                                    "Add a message (optional)"
                                                </button>
                                            </Show>
                                            <Show when=move || show_message.get()>
                                                <label class=label_class>"About Yourself / Message"</label>
                                                <textarea
                                                    class=format!("{input_class} min-h-[100px] resize-y")
                                                    placeholder="Employment, number of occupants, pets, etc."
                                                    prop:value=move || message.get()
                                                    on:input=move |ev| set_message.set(event_target_value(&ev))
                                                />
                                            </Show>
                                        </div>
                                    }
                                }
                            </div>

                            <div class="mt-8">
                                <button
                                    type="submit"
                                    class="w-full rounded-xl bg-stone-900 px-6 py-3 text-sm font-medium text-white hover:bg-stone-800 transition-all active:scale-[0.98] disabled:opacity-50 disabled:cursor-not-allowed"
                                    disabled=move || submitting.get()
                                >
                                    {move || if submitting.get() { "Submitting..." } else { "Submit Application" }}
                                </button>
                            </div>
                        </form>

                        // Screening disclosure
                        <div class="mt-8 border-t border-stone-200 pt-6">
                            <p class="text-[10px] font-semibold uppercase tracking-wider text-stone-400 mb-1.5">"Screening Disclosure"</p>
                            <p class="text-xs text-stone-400 leading-relaxed">
                                "Per WA law (RCW 59.18.257), this property does not accept reusable screening reports. "
                                "Applicants are screened via TransUnion SmartMove."
                            </p>
                        </div>
                    </div>
                </Show>
            </div>
        </div>
    }
}
