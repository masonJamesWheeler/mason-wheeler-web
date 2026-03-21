#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Payment;

#[component]
pub fn AdminPayments() -> impl IntoView {
    let (payments, set_payments) = signal::<Vec<Payment>>(vec![]);
    let (show_utility_form, set_show_utility_form) = signal(false);
    let (util_desc, set_util_desc) = signal(String::new());
    let (util_amount, set_util_amount) = signal(String::new());
    let (util_due, set_util_due) = signal(String::new());
    let (submitting, set_submitting) = signal(false);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/admin/payments").send().await {
                if let Ok(data) = resp.json::<Vec<Payment>>().await {
                    set_payments.set(data);
                }
            }
        });
    }

    let handle_add_utility = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        #[cfg(feature = "hydrate")]
        {
            let desc = util_desc.get();
            let amount: f64 = util_amount.get().parse().unwrap_or(0.0);
            let due = util_due.get();

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "description": desc,
                    "amount": amount,
                    "due_date": due,
                });

                if let Ok(resp) = gloo_net::http::Request::post("/api/admin/utilities")
                    .json(&body)
                    .unwrap()
                    .send()
                    .await
                {
                    if resp.ok() {
                        set_show_utility_form.set(false);
                        set_util_desc.set(String::new());
                        set_util_amount.set(String::new());
                        set_util_due.set(String::new());
                    }
                }
                set_submitting.set(false);
            });
        }
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-slate-900">"Payments & Billing"</h1>
                <button
                    class="bg-orange-600 hover:bg-orange-700 text-white font-medium px-4 py-2 rounded-lg transition-colors"
                    on:click=move |_| set_show_utility_form.update(|v| *v = !*v)
                >
                    {move || if show_utility_form.get() { "Cancel" } else { "Add Utility Charge" }}
                </button>
            </div>

            // Utility charge form
            <Show when=move || show_utility_form.get()>
                <form on:submit=handle_add_utility class="bg-white border border-slate-200 rounded-xl p-6 mb-6">
                    <h2 class="font-semibold text-slate-900 mb-4">"Add Utility Charge"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1">"Description"</label>
                            <input
                                type="text"
                                required=true
                                class="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 outline-none"
                                placeholder="e.g., Water/Sewer - March"
                                on:input=move |ev| set_util_desc.set(event_target_value(&ev))
                                prop:value=move || util_desc.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1">"Amount"</label>
                            <input
                                type="number"
                                step="0.01"
                                required=true
                                class="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 outline-none"
                                placeholder="0.00"
                                on:input=move |ev| set_util_amount.set(event_target_value(&ev))
                                prop:value=move || util_amount.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-slate-700 mb-1">"Due Date"</label>
                            <input
                                type="date"
                                required=true
                                class="w-full px-4 py-2 border border-slate-300 rounded-lg focus:ring-2 focus:ring-orange-500 outline-none"
                                on:input=move |ev| set_util_due.set(event_target_value(&ev))
                                prop:value=move || util_due.get()
                            />
                        </div>
                    </div>
                    <button
                        type="submit"
                        class="mt-4 bg-orange-600 hover:bg-orange-700 text-white font-medium px-6 py-2 rounded-lg transition-colors disabled:opacity-50"
                        disabled=move || submitting.get()
                    >
                        {move || if submitting.get() { "Adding..." } else { "Add Charge" }}
                    </button>
                </form>
            </Show>

            // Payment history
            <h2 class="text-lg font-semibold text-slate-900 mb-3">"All Payments"</h2>
            <Show
                when=move || !payments.get().is_empty()
                fallback=|| view! { <p class="text-slate-500">"No payments recorded yet."</p> }
            >
                <div class="bg-white border border-slate-200 rounded-xl overflow-hidden">
                    <table class="w-full text-sm">
                        <thead class="bg-slate-50">
                            <tr>
                                <th class="text-left px-4 py-3 font-medium text-slate-500">"Date"</th>
                                <th class="text-left px-4 py-3 font-medium text-slate-500">"Type"</th>
                                <th class="text-left px-4 py-3 font-medium text-slate-500">"Description"</th>
                                <th class="text-right px-4 py-3 font-medium text-slate-500">"Amount"</th>
                                <th class="text-right px-4 py-3 font-medium text-slate-500">"Status"</th>
                            </tr>
                        </thead>
                        <tbody class="divide-y divide-slate-100">
                            {move || payments.get().iter().map(|p| {
                                let date = p.paid_date.clone().or(p.due_date.clone()).unwrap_or_default();
                                let ptype = p.payment_type.clone();
                                let desc = p.description.clone().unwrap_or_default();
                                let amount = p.amount;
                                let status = p.status.clone();
                                let status_class = match status.as_str() {
                                    "completed" => "text-green-600 bg-green-50",
                                    "pending" => "text-yellow-600 bg-yellow-50",
                                    _ => "text-red-600 bg-red-50",
                                };
                                view! {
                                    <tr>
                                        <td class="px-4 py-3 text-slate-700">{date}</td>
                                        <td class="px-4 py-3 text-slate-700 capitalize">{ptype}</td>
                                        <td class="px-4 py-3 text-slate-700">{desc}</td>
                                        <td class="px-4 py-3 text-right font-medium text-slate-900">{format!("${:.2}", amount)}</td>
                                        <td class="px-4 py-3 text-right">
                                            <span class={format!("text-xs font-medium px-2 py-1 rounded-full capitalize {}", status_class)}>
                                                {status}
                                            </span>
                                        </td>
                                    </tr>
                                }
                            }).collect_view()}
                        </tbody>
                    </table>
                </div>
            </Show>
        </div>
    }
}
