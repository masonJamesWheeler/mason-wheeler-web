#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::MaintenanceRequest;

#[allow(unused)]
#[component]
pub fn TenantMaintenance() -> impl IntoView {
    let (requests, set_requests) = signal::<Vec<MaintenanceRequest>>(vec![]);
    let (loading, set_loading) = signal(true);
    let (show_form, set_show_form) = signal(false);
    let (title, set_title) = signal(String::new());
    let (description, set_description) = signal(String::new());
    let (submitting, set_submitting) = signal(false);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/maintenance").send().await {
                if let Ok(data) = resp.json::<Vec<MaintenanceRequest>>().await {
                    set_requests.set(data);
                }
            }
            set_loading.set(false);
        });
    }

    let handle_submit = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        #[cfg(feature = "hydrate")]
        {
            let title_val = title.get();
            let desc_val = description.get();

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "title": title_val,
                    "description": desc_val,
                });

                if let Ok(resp) = gloo_net::http::Request::post("/api/maintenance")
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(req) = resp.json::<MaintenanceRequest>().await {
                            set_requests.update(|r| r.insert(0, req));
                        }
                        set_show_form.set(false);
                        set_title.set(String::new());
                        set_description.set(String::new());
                    }
                }
                set_submitting.set(false);
            });
        }
    };

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10 font-['Inter',sans-serif]">
            // Header
            <div class="flex items-center justify-between mb-8">
                <div>
                    <h1 class="section-title text-3xl font-bold text-stone-900 tracking-tight">"Maintenance"</h1>
                    <p class="text-stone-500 text-sm mt-1">"Track and manage your maintenance requests"</p>
                </div>
                <button
                    class="btn-primary"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    {move || if show_form.get() { "Cancel" } else { "New Request" }}
                </button>
            </div>

            // New request form
            <Show when=move || show_form.get()>
                <form on:submit=handle_submit class="card mb-8">
                    <h2 class="text-lg font-semibold text-stone-900 mb-6">"Submit Maintenance Request"</h2>
                    <div class="space-y-5">
                        <div>
                            <label class="stat-label block text-sm font-medium text-stone-600 mb-1.5">"Title"</label>
                            <input
                                type="text"
                                required=true
                                class="input"
                                placeholder="e.g., Leaky faucet in kitchen"
                                on:input=move |ev| set_title.set(event_target_value(&ev))
                                prop:value=move || title.get()
                            />
                        </div>
                        <div>
                            <label class="stat-label block text-sm font-medium text-stone-600 mb-1.5">"Description"</label>
                            <textarea
                                required=true
                                rows="4"
                                class="input"
                                placeholder="Describe the issue in detail..."
                                on:input=move |ev| set_description.set(event_target_value(&ev))
                                prop:value=move || description.get()
                            />
                        </div>
                        <div class="flex items-center gap-3 pt-2">
                            <button
                                type="submit"
                                class="btn-accent"
                                disabled=move || submitting.get()
                            >
                                {move || if submitting.get() { "Submitting..." } else { "Submit Request" }}
                            </button>
                            <button
                                type="button"
                                class="btn-secondary"
                                on:click=move |_| set_show_form.set(false)
                            >
                                "Cancel"
                            </button>
                        </div>
                    </div>
                </form>
            </Show>

            // Requests list
            <Show
                when=move || !loading.get()
                fallback=|| view! {
                    <div class="flex items-center justify-center py-16">
                        <div class="flex items-center gap-3 text-stone-400">
                            <svg class="animate-spin h-5 w-5" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
                                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
                                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8v4a4 4 0 00-4 4H4z"></path>
                            </svg>
                            <span class="text-sm font-medium">"Loading requests..."</span>
                        </div>
                    </div>
                }
            >
                <Show
                    when=move || !requests.get().is_empty()
                    fallback=|| view! {
                        <div class="text-center py-20">
                            <div class="inline-flex items-center justify-center w-16 h-16 rounded-full bg-stone-100 mb-5">
                                <svg class="w-8 h-8 text-stone-400" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M11.42 15.17l-5.384-3.077A1.5 1.5 0 015 10.768V5.25A2.25 2.25 0 017.25 3h9.5A2.25 2.25 0 0119 5.25v5.518a1.5 1.5 0 01-1.036 1.425l-5.384 3.077a1.5 1.5 0 01-1.16 0z" />
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 15.75v3.75" />
                                </svg>
                            </div>
                            <p class="text-lg font-medium text-stone-700">"No maintenance requests yet"</p>
                            <p class="text-sm text-stone-400 mt-2 max-w-sm mx-auto">
                                "Everything looking good? If something needs fixing, click "
                                <span class="font-medium text-amber-600">"New Request"</span>
                                " to let us know."
                            </p>
                        </div>
                    }
                >
                    <div class="space-y-4">
                        {move || requests.get().iter().map(|req| {
                            let title = req.title.clone();
                            let desc = req.description.clone();
                            let status = req.status.clone();
                            let date = req.created_at.clone();
                            let badge_class = match status.as_str() {
                                "submitted" => "badge-warning",
                                "in_progress" => "badge-info",
                                "completed" => "badge-success",
                                _ => "badge-info",
                            };
                            let status_label = status.replace('_', " ");
                            view! {
                                <div class="card card-hover">
                                    <div class="flex items-start justify-between gap-4">
                                        <div class="min-w-0 flex-1">
                                            <div class="flex items-center gap-3 mb-2">
                                                <h3 class="font-semibold text-stone-900 truncate">{title}</h3>
                                                <span class={badge_class}>{status_label}</span>
                                            </div>
                                            <p class="text-sm text-stone-500 leading-relaxed">{desc}</p>
                                            <p class="text-xs text-stone-400 mt-3 flex items-center gap-1.5">
                                                <svg class="w-3.5 h-3.5" fill="none" stroke="currentColor" viewBox="0 0 24 24" stroke-width="1.5">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                </svg>
                                                {format!("Submitted {}", date)}
                                            </p>
                                        </div>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </Show>
            </Show>
        </div>
    }
}
