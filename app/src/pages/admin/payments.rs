#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Payment;

#[allow(unused)]
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
        <div class="max-w-5xl mx-auto px-6 py-10">
            <div class="flex items-center justify-between mb-8">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-stone-900">"Payments & Billing"</h1>
                    <p class="text-stone-500 mt-1">"Manage rent, utilities, and payment history"</p>
                </div>
                <button
                    class="btn-primary"
                    on:click=move |_| set_show_utility_form.update(|v| *v = !*v)
                >
                    {move || if show_utility_form.get() { "Cancel" } else { "Add Utility Charge" }}
                </button>
            </div>

            // Utility charge form
            <Show when=move || show_utility_form.get()>
                <form on:submit=handle_add_utility class="card mb-8">
                    <h2 class="section-title">"Add Utility Charge"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
                        <div>
                            <label class="block text-sm font-medium text-stone-600 mb-1.5">"Description"</label>
                            <input
                                type="text"
                                required=true
                                class="input"
                                placeholder="e.g., Water/Sewer - March"
                                on:input=move |ev| set_util_desc.set(event_target_value(&ev))
                                prop:value=move || util_desc.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-stone-600 mb-1.5">"Amount"</label>
                            <input
                                type="number"
                                step="0.01"
                                required=true
                                class="input"
                                placeholder="0.00"
                                on:input=move |ev| set_util_amount.set(event_target_value(&ev))
                                prop:value=move || util_amount.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-stone-600 mb-1.5">"Due Date"</label>
                            <input
                                type="date"
                                required=true
                                class="input"
                                on:input=move |ev| set_util_due.set(event_target_value(&ev))
                                prop:value=move || util_due.get()
                            />
                        </div>
                    </div>
                    <div class="mt-6 flex justify-end">
                        <button
                            type="submit"
                            class="btn-accent"
                            disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Adding..." } else { "Add Charge" }}
                        </button>
                    </div>
                </form>
            </Show>

            // Payment history
            <h2 class="section-title">"All Payments"</h2>
            <Show
                when=move || !payments.get().is_empty()
                fallback=|| view! {
                    <div class="card text-center py-12">
                        <p class="text-stone-400 text-sm">"No payments recorded yet."</p>
                    </div>
                }
            >
                <div class="card !p-0 overflow-hidden">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="border-b border-stone-200/60">
                                <th class="text-left px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Date"</th>
                                <th class="text-left px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Type"</th>
                                <th class="text-left px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Description"</th>
                                <th class="text-right px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Amount"</th>
                                <th class="text-right px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Status"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || payments.get().iter().map(|p| {
                                let date = p.paid_date.clone().or(p.due_date.clone()).unwrap_or_default();
                                let ptype = p.payment_type.clone();
                                let desc = p.description.clone().unwrap_or_default();
                                let amount = p.amount;
                                let status = p.status.clone();
                                let badge_class = match status.as_str() {
                                    "completed" => "badge-success",
                                    "pending" => "badge-warning",
                                    _ => "badge-error",
                                };
                                view! {
                                    <tr class="border-b border-stone-200/60 last:border-0 hover:bg-stone-50/50 transition-colors">
                                        <td class="px-5 py-3.5 text-stone-600">{date}</td>
                                        <td class="px-5 py-3.5 text-stone-600 capitalize">{ptype}</td>
                                        <td class="px-5 py-3.5 text-stone-700">{desc}</td>
                                        <td class="px-5 py-3.5 text-right font-semibold text-stone-900">{format!("${:.2}", amount)}</td>
                                        <td class="px-5 py-3.5 text-right">
                                            <span class={format!("capitalize {}", badge_class)}>
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
