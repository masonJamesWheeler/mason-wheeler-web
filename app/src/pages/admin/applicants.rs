#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Applicant;

#[component]
pub fn AdminApplicants() -> impl IntoView {
    let (applicants, set_applicants) = signal::<Vec<Applicant>>(vec![]);
    let (expanded_id, set_expanded_id) = signal::<Option<String>>(None);
    let (loading, set_loading) = signal(true);

    #[cfg(feature = "hydrate")]
    {
        let set_applicants = set_applicants.clone();
        let set_loading = set_loading.clone();
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/admin/applicants").send().await {
                if let Ok(data) = resp.json::<Vec<Applicant>>().await {
                    set_applicants.set(data);
                }
            }
            set_loading.set(false);
        });
    }

    let toggle_expand = move |id: String| {
        set_expanded_id.update(|current| {
            if current.as_ref() == Some(&id) {
                *current = None;
            } else {
                *current = Some(id);
            }
        });
    };

    let handle_status_change = move |id: String, new_status: String| {
        #[cfg(feature = "hydrate")]
        {
            let set_applicants = set_applicants.clone();
            leptos::task::spawn_local(async move {
                let body = serde_json::json!({"status": new_status});
                if let Ok(resp) = gloo_net::http::Request::patch(&format!("/api/admin/applicants/{}", id))
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        set_applicants.update(|apps| {
                            if let Some(app) = apps.iter_mut().find(|a| a.id == id) {
                                app.status = new_status.clone();
                            }
                        });
                    }
                }
            });
        }
    };

    let handle_notes_save = move |id: String, notes: String| {
        #[cfg(feature = "hydrate")]
        {
            let set_applicants = set_applicants.clone();
            leptos::task::spawn_local(async move {
                let body = serde_json::json!({"notes": notes});
                if let Ok(resp) = gloo_net::http::Request::patch(&format!("/api/admin/applicants/{}", id))
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        set_applicants.update(|apps| {
                            if let Some(app) = apps.iter_mut().find(|a| a.id == id) {
                                app.notes = Some(notes.clone());
                            }
                        });
                    }
                }
            });
        }
    };

    let handle_delete = move |id: String| {
        #[cfg(feature = "hydrate")]
        {
            let set_applicants = set_applicants.clone();
            leptos::task::spawn_local(async move {
                if let Ok(resp) = gloo_net::http::Request::delete(&format!("/api/admin/applicants/{}", id))
                    .send()
                    .await
                {
                    if resp.ok() {
                        set_applicants.update(|apps| apps.retain(|a| a.id != id));
                    }
                }
            });
        }
    };

    view! {
        <div class="mx-auto max-w-5xl px-6 py-8">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-xl font-semibold text-stone-900">"Applications"</h1>
                <span class="text-sm text-stone-500">
                    {move || {
                        let count = applicants.get().len();
                        format!("{} applicant{}", count, if count == 1 { "" } else { "s" })
                    }}
                </span>
            </div>

            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-20">
                    <p class="text-sm text-stone-500">"Loading applicants..."</p>
                </div>
            </Show>

            <Show when=move || !loading.get() && applicants.get().is_empty()>
                <div class="rounded-xl border border-stone-200 bg-white p-12 text-center">
                    <p class="text-stone-500">"No applications yet."</p>
                </div>
            </Show>

            <Show when=move || !loading.get() && !applicants.get().is_empty()>
                <div class="overflow-hidden rounded-xl border border-stone-200 bg-white shadow-sm">
                    // Table header
                    <div class="hidden sm:grid sm:grid-cols-12 gap-4 px-6 py-3 bg-stone-50 border-b border-stone-200 text-xs font-medium uppercase tracking-wider text-stone-500">
                        <div class="col-span-2">"Name"</div>
                        <div class="col-span-3">"Email"</div>
                        <div class="col-span-2">"Phone"</div>
                        <div class="col-span-2">"Move-in"</div>
                        <div class="col-span-1">"Status"</div>
                        <div class="col-span-2">"Submitted"</div>
                    </div>

                    <For
                        each=move || applicants.get()
                        key=|a| a.id.clone()
                        children=move |app: Applicant| {
                            let app_id = app.id.clone();
                            let app_id_toggle = app.id.clone();
                            let app_id_status = app.id.clone();
                            let app_id_notes = app.id.clone();
                            let app_id_delete = app.id.clone();
                            let app_status = app.status.clone();
                            let app_status_1 = app.status.clone();
                            let app_status_2 = app.status.clone();
                            let app_status_3 = app.status.clone();
                            let app_status_4 = app.status.clone();
                            let app_notes = app.notes.clone().unwrap_or_default();
                            let app_message = app.message.clone().unwrap_or_default();

                            let status_badge = {
                                let s = app.status.clone();
                                let (bg, text) = match s.as_str() {
                                    "new" => ("bg-blue-100 text-blue-700", "New"),
                                    "screening" => ("bg-yellow-100 text-yellow-700", "Screening"),
                                    "approved" => ("bg-green-100 text-green-700", "Approved"),
                                    "denied" => ("bg-red-100 text-red-700", "Denied"),
                                    "lease_signed" => ("bg-green-100 text-green-700", "Lease Signed"),
                                    _ => ("bg-stone-100 text-stone-700", "Unknown"),
                                };
                                view! {
                                    <span class=format!("inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium {bg} {text}")>
                                        {text}
                                    </span>
                                }
                            };

                            let is_expanded = Signal::derive({
                                let id = app_id.clone();
                                move || expanded_id.get().as_ref() == Some(&id)
                            });

                            // Format created_at for display
                            let created_display = app.created_at.get(..10).unwrap_or(&app.created_at).to_string();

                            view! {
                                <div class="border-b border-stone-100 last:border-b-0">
                                    // Row
                                    <div
                                        class="grid grid-cols-1 sm:grid-cols-12 gap-2 sm:gap-4 px-6 py-4 cursor-pointer hover:bg-stone-50 transition-colors"
                                        on:click={
                                            let id = app_id_toggle.clone();
                                            move |_| toggle_expand(id.clone())
                                        }
                                    >
                                        <div class="sm:col-span-2 font-medium text-stone-900 text-sm">{app.name.clone()}</div>
                                        <div class="sm:col-span-3 text-sm text-stone-600 truncate">{app.email.clone()}</div>
                                        <div class="sm:col-span-2 text-sm text-stone-500">{app.phone.clone().unwrap_or_else(|| "—".to_string())}</div>
                                        <div class="sm:col-span-2 text-sm text-stone-500">{app.desired_move_in.clone().unwrap_or_else(|| "—".to_string())}</div>
                                        <div class="sm:col-span-1">{status_badge}</div>
                                        <div class="sm:col-span-2 text-sm text-stone-400">{created_display}</div>
                                    </div>

                                    // Expanded detail panel
                                    <Show when=move || is_expanded.get()>
                                        {
                                            let app_message_display = app_message.clone();
                                            let app_message_check = app_message.clone();
                                            let app_status = app_status.clone();
                                            let app_status_1 = app_status_1.clone();
                                            let app_status_2 = app_status_2.clone();
                                            let app_status_3 = app_status_3.clone();
                                            let app_status_4 = app_status_4.clone();
                                            let app_id_status = app_id_status.clone();
                                            let app_id_notes = app_id_notes.clone();
                                            let app_id_delete = app_id_delete.clone();
                                            let app_notes = app_notes.clone();
                                            view! {
                                        <div class="border-t border-stone-100 bg-stone-50 px-6 py-5 space-y-4">
                                            // Message
                                            <Show when={
                                                move || !app_message_check.is_empty()
                                            }>
                                                <div>
                                                    <p class="text-xs font-medium uppercase tracking-wider text-stone-400 mb-1">"Message"</p>
                                                    <p class="text-sm text-stone-700 whitespace-pre-wrap">{app_message_display.clone()}</p>
                                                </div>
                                            </Show>

                                            // Status dropdown
                                            <div>
                                                <label class="text-xs font-medium uppercase tracking-wider text-stone-400 mb-1 block">"Status"</label>
                                                <select
                                                    class="rounded-lg border border-stone-300 px-3 py-2 text-sm text-stone-900 focus:border-stone-500 focus:outline-none focus:ring-1 focus:ring-stone-500"
                                                    on:change={
                                                        let id = app_id_status.clone();
                                                        move |ev| {
                                                            let val = event_target_value(&ev);
                                                            handle_status_change(id.clone(), val);
                                                        }
                                                    }
                                                >
                                                    <option value="new" selected=move || app_status == "new">"New"</option>
                                                    <option value="screening" selected=move || app_status_1 == "screening">"Screening"</option>
                                                    <option value="approved" selected=move || app_status_2 == "approved">"Approved"</option>
                                                    <option value="denied" selected=move || app_status_3 == "denied">"Denied"</option>
                                                    <option value="lease_signed" selected=move || app_status_4 == "lease_signed">"Lease Signed"</option>
                                                </select>
                                            </div>

                                            // Notes
                                            <div>
                                                <label class="text-xs font-medium uppercase tracking-wider text-stone-400 mb-1 block">"Notes"</label>
                                                {
                                                    let (notes_val, set_notes_val) = signal(app_notes.clone());
                                                    let notes_id = app_id_notes.clone();
                                                    view! {
                                                        <textarea
                                                            class="w-full rounded-lg border border-stone-300 px-3 py-2 text-sm text-stone-900 min-h-[80px] resize-y focus:border-stone-500 focus:outline-none focus:ring-1 focus:ring-stone-500"
                                                            prop:value=move || notes_val.get()
                                                            on:input=move |ev| set_notes_val.set(event_target_value(&ev))
                                                        />
                                                        <button
                                                            class="mt-2 rounded-lg bg-stone-800 px-4 py-1.5 text-xs font-medium text-white hover:bg-stone-700 transition-colors"
                                                            on:click={
                                                                let id = notes_id.clone();
                                                                move |_| handle_notes_save(id.clone(), notes_val.get())
                                                            }
                                                        >"Save Notes"</button>
                                                    }
                                                }
                                            </div>

                                            // Action buttons
                                            <div class="flex items-center gap-3 pt-2">
                                                <a
                                                    href="https://rentals-secure.mysmartmove.com"
                                                    target="_blank"
                                                    rel="noopener noreferrer"
                                                    class="inline-flex items-center gap-1.5 rounded-lg bg-blue-600 px-4 py-2 text-xs font-medium text-white hover:bg-blue-700 transition-colors"
                                                >
                                                    "Screen via SmartMove"
                                                    <svg class="h-3.5 w-3.5" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                                        <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25" />
                                                    </svg>
                                                </a>
                                                <button
                                                    class="inline-flex items-center gap-1.5 rounded-lg border border-red-200 px-4 py-2 text-xs font-medium text-red-600 hover:bg-red-50 transition-colors"
                                                    on:click={
                                                        let id = app_id_delete.clone();
                                                        move |_| handle_delete(id.clone())
                                                    }
                                                >
                                                    "Delete"
                                                </button>
                                            </div>
                                        </div>
                                            }
                                        }
                                    </Show>
                                </div>
                            }
                        }
                    />
                </div>
            </Show>
        </div>
    }
}
