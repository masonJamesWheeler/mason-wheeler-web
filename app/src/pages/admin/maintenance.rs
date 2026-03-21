#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::MaintenanceRequest;

#[allow(unused)]
#[component]
pub fn AdminMaintenance() -> impl IntoView {
    let (requests, set_requests) = signal::<Vec<MaintenanceRequest>>(vec![]);
    let (loading, set_loading) = signal(true);

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

    let update_status = move |id: String, new_status: String| {
        #[cfg(feature = "hydrate")]
        {
            let id_clone = id.clone();
            let status_clone = new_status.clone();
            leptos::task::spawn_local(async move {
                let body = serde_json::json!({ "status": status_clone });
                if let Ok(resp) = gloo_net::http::Request::patch(&format!("/api/maintenance/{}", id_clone))
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        set_requests.update(|reqs| {
                            if let Some(req) = reqs.iter_mut().find(|r| r.id == id) {
                                req.status = new_status.clone();
                            }
                        });
                    }
                }
            });
        }
    };

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10">
            <h1 class="text-3xl font-bold tracking-tight text-stone-900 mb-1">"Maintenance Requests"</h1>
            <p class="text-stone-500 mb-8">"Review and manage tenant maintenance requests"</p>

            <Show
                when=move || !loading.get()
                fallback=|| view! {
                    <div class="card text-center py-12">
                        <p class="text-stone-400 text-sm">"Loading requests..."</p>
                    </div>
                }
            >
                <Show
                    when=move || !requests.get().is_empty()
                    fallback=|| view! {
                        <div class="card text-center py-16">
                            <svg class="w-12 h-12 text-stone-300 mx-auto mb-3" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                            </svg>
                            <p class="text-stone-500 font-medium">"No maintenance requests"</p>
                            <p class="text-stone-400 text-sm mt-1">"All caught up!"</p>
                        </div>
                    }
                >
                    <div class="space-y-4">
                        {move || requests.get().iter().map(|req| {
                            let id = req.id.clone();
                            let title = req.title.clone();
                            let desc = req.description.clone();
                            let status = req.status.clone();
                            let date = req.created_at.clone();

                            let id_progress = id.clone();
                            let id_complete = id.clone();

                            let badge_class = match status.as_str() {
                                "submitted" => "badge-warning",
                                "in_progress" => "badge-info",
                                "completed" => "badge-success",
                                _ => "badge-neutral",
                            };

                            view! {
                                <div class="card">
                                    <div class="flex items-start justify-between mb-4">
                                        <div class="min-w-0 flex-1">
                                            <h3 class="font-semibold text-stone-900 text-lg">{title}</h3>
                                            <p class="text-sm text-stone-500 mt-1.5 leading-relaxed">{desc}</p>
                                            <p class="text-xs text-stone-400 mt-2.5">
                                                {format!("Submitted {}", date)}
                                            </p>
                                        </div>
                                        <span class={format!("capitalize ml-4 shrink-0 {}", badge_class)}>
                                            {status.replace('_', " ")}
                                        </span>
                                    </div>
                                    <div class="flex gap-2 pt-3 border-t border-stone-200/60">
                                        <button
                                            class="btn-secondary text-sm"
                                            on:click=move |_| update_status(id_progress.clone(), "in_progress".to_string())
                                        >
                                            "Mark In Progress"
                                        </button>
                                        <button
                                            class="btn-accent text-sm"
                                            on:click=move |_| update_status(id_complete.clone(), "completed".to_string())
                                        >
                                            "Mark Completed"
                                        </button>
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
