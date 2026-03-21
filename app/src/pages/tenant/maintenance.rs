use leptos::prelude::*;
use mason_wheeler_shared::MaintenanceRequest;

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
        <div class="max-w-4xl mx-auto px-4 py-8">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-slate-900">"Maintenance"</h1>
                <button
                    class="bg-orange-600 hover:bg-orange-700 text-white font-medium px-4 py-2 rounded-lg transition-colors"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    {move || if show_form.get() { "Cancel" } else { "New Request" }}
                </button>
            </div>

            // New request form
            <Show when=move || show_form.get()>
                <form on:submit=handle_submit class="bg-white border border-slate-200 rounded-xl p-6 mb-6">
                    <h2 class="font-semibold text-slate-900 mb-4">"Submit Maintenance Request"</h2>
                    <div class="space-y-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1">"Title"</label>
                            <input
                                type="text"
                                required=true
                                class="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none"
                                placeholder="e.g., Leaky faucet in kitchen"
                                on:input=move |ev| set_title.set(event_target_value(&ev))
                                prop:value=move || title.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1">"Description"</label>
                            <textarea
                                required=true
                                rows="4"
                                class="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 focus:border-orange-500 outline-none"
                                placeholder="Describe the issue in detail..."
                                on:input=move |ev| set_description.set(event_target_value(&ev))
                                prop:value=move || description.get()
                            />
                        </div>
                        <button
                            type="submit"
                            class="bg-orange-600 hover:bg-orange-700 text-white font-medium px-6 py-2 rounded-lg transition-colors disabled:opacity-50"
                            disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Submitting..." } else { "Submit Request" }}
                        </button>
                    </div>
                </form>
            </Show>

            // Requests list
            <Show
                when=move || !loading.get()
                fallback=|| view! { <p class="text-slate-500">"Loading..."</p> }
            >
                <Show
                    when=move || !requests.get().is_empty()
                    fallback=|| view! {
                        <div class="text-center py-12 text-slate-500">
                            <p class="text-lg">"No maintenance requests yet."</p>
                            <p class="text-sm mt-1">"Click 'New Request' if you need something fixed."</p>
                        </div>
                    }
                >
                    <div class="space-y-3">
                        {move || requests.get().iter().map(|req| {
                            let title = req.title.clone();
                            let desc = req.description.clone();
                            let status = req.status.clone();
                            let date = req.created_at.clone();
                            let status_class = match status.as_str() {
                                "submitted" => "text-yellow-700 bg-yellow-50 border-yellow-200",
                                "in_progress" => "text-blue-700 bg-blue-50 border-blue-200",
                                "completed" => "text-green-700 bg-green-50 border-green-200",
                                _ => "text-slate-700 bg-slate-50 border-slate-200",
                            };
                            view! {
                                <div class={format!("border rounded-xl p-4 {}", status_class)}>
                                    <div class="flex items-start justify-between">
                                        <div>
                                            <p class="font-medium">{title}</p>
                                            <p class="text-sm mt-1 opacity-80">{desc}</p>
                                            <p class="text-xs mt-2 opacity-60">{format!("Submitted {}", date)}</p>
                                        </div>
                                        <span class="text-xs font-medium px-2 py-1 rounded-full capitalize bg-white/50">
                                            {status.replace('_', " ")}
                                        </span>
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
