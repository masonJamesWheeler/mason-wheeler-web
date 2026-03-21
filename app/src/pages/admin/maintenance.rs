#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::MaintenanceRequest;

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
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-slate-900 mb-6">"Maintenance Requests"</h1>

            <Show
                when=move || !loading.get()
                fallback=|| view! { <p class="text-slate-500">"Loading..."</p> }
            >
                <Show
                    when=move || !requests.get().is_empty()
                    fallback=|| view! {
                        <p class="text-slate-500 text-center py-12">"No maintenance requests."</p>
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

                            view! {
                                <div class="bg-white border border-slate-200 rounded-xl p-6">
                                    <div class="flex items-start justify-between mb-3">
                                        <div>
                                            <h3 class="font-semibold text-slate-900">{title}</h3>
                                            <p class="text-sm text-slate-500 mt-1">{desc}</p>
                                            <p class="text-xs text-slate-400 mt-2">{format!("Submitted {}", date)}</p>
                                        </div>
                                        <span class={format!("text-xs font-medium px-2 py-1 rounded-full capitalize {}",
                                            match status.as_str() {
                                                "submitted" => "text-yellow-700 bg-yellow-50",
                                                "in_progress" => "text-blue-700 bg-blue-50",
                                                "completed" => "text-green-700 bg-green-50",
                                                _ => "text-slate-700 bg-slate-50",
                                            }
                                        )}>
                                            {status.replace('_', " ")}
                                        </span>
                                    </div>
                                    <div class="flex gap-2">
                                        <button
                                            class="text-xs bg-blue-100 hover:bg-blue-200 text-blue-700 px-3 py-1.5 rounded-lg transition-colors"
                                            on:click=move |_| update_status(id_progress.clone(), "in_progress".to_string())
                                        >
                                            "Mark In Progress"
                                        </button>
                                        <button
                                            class="text-xs bg-green-100 hover:bg-green-200 text-green-700 px-3 py-1.5 rounded-lg transition-colors"
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
