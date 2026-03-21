#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::{Payment, UtilityCharge};

#[component]
pub fn TenantPayments() -> impl IntoView {
    let (payments, set_payments) = signal::<Vec<Payment>>(vec![]);
    let (utilities, set_utilities) = signal::<Vec<UtilityCharge>>(vec![]);
    let (loading, set_loading) = signal(true);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/tenant/payments").send().await {
                if let Ok(data) = resp.json::<Vec<Payment>>().await {
                    set_payments.set(data);
                }
            }
            if let Ok(resp) = gloo_net::http::Request::get("/api/tenant/utilities").send().await {
                if let Ok(data) = resp.json::<Vec<UtilityCharge>>().await {
                    set_utilities.set(data);
                }
            }
            set_loading.set(false);
        });
    }

    let handle_pay_rent = move |_| {
        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async move {
                if let Ok(resp) = gloo_net::http::Request::post("/api/payments/create-checkout")
                    .json(&serde_json::json!({ "type": "rent", "amount": 3250.0 }))
                    .unwrap()
                    .send()
                    .await
                {
                    if let Ok(data) = resp.json::<serde_json::Value>().await {
                        if let Some(url) = data.get("url").and_then(|u| u.as_str()) {
                            if let Some(window) = web_sys::window() {
                                let _ = window.location().set_href(url);
                            }
                        }
                    }
                }
            });
        }
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <div class="flex items-center justify-between mb-6">
                <h1 class="text-2xl font-bold text-slate-900">"Payments"</h1>
                <button
                    class="bg-orange-600 hover:bg-orange-700 text-white font-medium px-6 py-2 rounded-lg transition-colors"
                    on:click=handle_pay_rent
                >
                    "Pay Rent — $3,250"
                </button>
            </div>

            // Outstanding utility charges
            <Show when=move || !utilities.get().is_empty()>
                <div class="mb-8">
                    <h2 class="text-lg font-semibold text-slate-900 mb-3">"Outstanding Charges"</h2>
                    <div class="space-y-2">
                        {move || utilities.get().iter().filter(|u| !u.paid).map(|charge| {
                            let desc = charge.description.clone();
                            let amount = charge.amount;
                            let due = charge.due_date.clone();
                            view! {
                                <div class="flex items-center justify-between bg-yellow-50 border border-yellow-200 rounded-lg p-4">
                                    <div>
                                        <p class="font-medium text-slate-900">{desc}</p>
                                        <p class="text-sm text-slate-500">{format!("Due {}", due)}</p>
                                    </div>
                                    <div class="flex items-center gap-4">
                                        <p class="font-semibold text-slate-900">{format!("${:.2}", amount)}</p>
                                        <button class="bg-orange-600 hover:bg-orange-700 text-white text-sm px-4 py-2 rounded-lg transition-colors">
                                            "Pay"
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>

            // Payment history
            <h2 class="text-lg font-semibold text-slate-900 mb-3">"Payment History"</h2>
            <Show
                when=move || !loading.get()
                fallback=|| view! { <p class="text-slate-500">"Loading..."</p> }
            >
                <Show
                    when=move || !payments.get().is_empty()
                    fallback=|| view! { <p class="text-slate-500">"No payments yet."</p> }
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
            </Show>

            // Payment note
            <div class="mt-6 bg-slate-50 rounded-lg p-4 text-sm text-slate-600">
                <p class="font-medium text-slate-700 mb-1">"Payment Information"</p>
                <ul class="list-disc list-inside space-y-1">
                    <li>"Rent is due on the 1st of each month"</li>
                    <li>"A 5-day grace period applies to electronic payments per WA state law"</li>
                    <li>"Bank transfer (ACH) payments are preferred and have lower processing fees"</li>
                    <li>"You may also pay by check or money order — contact your landlord for details"</li>
                </ul>
            </div>
        </div>
    }
}
