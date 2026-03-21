#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::{MaintenanceMessage, MaintenanceRequest};

#[allow(unused)]
#[component]
pub fn AdminMaintenance() -> impl IntoView {
    let (requests, set_requests) = signal::<Vec<MaintenanceRequest>>(vec![]);
    let (loading, set_loading) = signal(true);
    let (expanded_id, set_expanded_id) = signal::<Option<String>>(None);
    let (messages, set_messages) = signal::<Vec<MaintenanceMessage>>(vec![]);
    let (new_message, set_new_message) = signal(String::new());
    let (sending_message, set_sending_message) = signal(false);

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

    let toggle_messages = move |id: String| {
        if expanded_id.get().as_deref() == Some(&id) {
            set_expanded_id.set(None);
            set_messages.set(vec![]);
        } else {
            set_expanded_id.set(Some(id.clone()));
            set_messages.set(vec![]);
            set_new_message.set(String::new());

            #[cfg(feature = "hydrate")]
            {
                let id_clone = id.clone();
                leptos::task::spawn_local(async move {
                    if let Ok(resp) = gloo_net::http::Request::get(&format!("/api/maintenance/{}/messages", id_clone))
                        .send()
                        .await
                    {
                        if let Ok(data) = resp.json::<Vec<MaintenanceMessage>>().await {
                            set_messages.set(data);
                        }
                    }
                });
            }
        }
    };

    let send_message = move |request_id: String| {
        let msg = new_message.get();
        if msg.trim().is_empty() {
            return;
        }
        set_sending_message.set(true);

        #[cfg(feature = "hydrate")]
        {
            let msg_val = msg.clone();
            let req_id = request_id.clone();
            leptos::task::spawn_local(async move {
                let body = serde_json::json!({ "message": msg_val });
                if let Ok(resp) = gloo_net::http::Request::post(&format!("/api/maintenance/{}/messages", req_id))
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        if let Ok(msg) = resp.json::<MaintenanceMessage>().await {
                            set_messages.update(|m| m.push(msg));
                        }
                        set_new_message.set(String::new());
                    }
                }
                set_sending_message.set(false);
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
                            let id_toggle = id.clone();
                            let id_send = id.clone();

                            let badge_class = match status.as_str() {
                                "submitted" => "badge-warning",
                                "in_progress" => "badge-info",
                                "completed" => "badge-success",
                                _ => "badge-neutral",
                            };

                            let is_expanded = expanded_id.get().as_deref() == Some(&id);

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
                                        <button
                                            class="btn-secondary text-sm ml-auto"
                                            on:click=move |_| toggle_messages(id_toggle.clone())
                                        >
                                            {if is_expanded { "Hide Messages" } else { "View Messages" }}
                                        </button>
                                    </div>

                                    <Show when=move || expanded_id.get().as_deref() == Some(&id_send)>
                                        <div class="mt-4 pt-4 border-t border-stone-200/60">
                                            <h4 class="text-sm font-semibold text-stone-700 mb-3">"Messages"</h4>

                                            <Show
                                                when=move || !messages.get().is_empty()
                                                fallback=|| view! {
                                                    <p class="text-xs text-stone-400 mb-3">"No messages yet. Start the conversation below."</p>
                                                }
                                            >
                                                <div class="space-y-3 mb-4 max-h-64 overflow-y-auto">
                                                    {move || messages.get().iter().map(|msg| {
                                                        let sender = msg.user_name.clone().unwrap_or_else(|| "Unknown".to_string());
                                                        let text = msg.message.clone();
                                                        let time = msg.created_at.clone();
                                                        view! {
                                                            <div class="rounded-lg bg-stone-50 p-3">
                                                                <div class="flex items-center justify-between mb-1">
                                                                    <span class="text-xs font-semibold text-stone-700">{sender}</span>
                                                                    <span class="text-xs text-stone-400">{time}</span>
                                                                </div>
                                                                <p class="text-sm text-stone-600">{text}</p>
                                                            </div>
                                                        }
                                                    }).collect_view()}
                                                </div>
                                            </Show>

                                            <div class="flex gap-2">
                                                <input
                                                    type="text"
                                                    class="input flex-1"
                                                    placeholder="Type a message..."
                                                    on:input=move |ev| set_new_message.set(event_target_value(&ev))
                                                    prop:value=move || new_message.get()
                                                />
                                                <button
                                                    type="button"
                                                    class="btn-accent text-sm"
                                                    disabled=move || sending_message.get()
                                                    on:click={
                                                        let req_id = id.clone();
                                                        move |_| send_message(req_id.clone())
                                                    }
                                                >
                                                    {move || if sending_message.get() { "Sending..." } else { "Send" }}
                                                </button>
                                            </div>
                                        </div>
                                    </Show>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </Show>
            </Show>
        </div>
    }
}
