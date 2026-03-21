#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::{Payment, UtilityCharge};

#[allow(unused)]
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
        <div class="max-w-5xl mx-auto px-6 py-10" style="font-family: 'Inter', sans-serif;">

            // Page header with prominent Pay Rent CTA
            <div class="flex items-center justify-between mb-10">
                <div>
                    <h1 class="text-2xl font-bold text-stone-900 tracking-tight">"Payments"</h1>
                    <p class="text-sm text-stone-400 mt-1">"Manage rent and utility payments"</p>
                </div>
                <button
                    class="btn-accent"
                    on:click=handle_pay_rent
                >
                    <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M4 4a2 2 0 00-2 2v4a2 2 0 002 2V6h10a2 2 0 00-2-2H4zm2 6a2 2 0 012-2h8a2 2 0 012 2v4a2 2 0 01-2 2H8a2 2 0 01-2-2v-4zm6 4a2 2 0 100-4 2 2 0 000 4z" clip-rule="evenodd"/>
                    </svg>
                    "Pay Rent \u{2014} $3,250"
                </button>
            </div>

            // Outstanding utility charges
            <Show when=move || !utilities.get().is_empty()>
                <div class="mb-10">
                    <h2 class="section-title mb-4">"Outstanding Charges"</h2>
                    <div class="space-y-3">
                        {move || utilities.get().iter().filter(|u| !u.paid).map(|charge| {
                            let desc = charge.description.clone();
                            let amount = charge.amount;
                            let due = charge.due_date.clone();
                            view! {
                                <div class="card card-hover flex items-center justify-between p-5">
                                    <div class="flex items-center gap-4">
                                        <div class="w-10 h-10 rounded-xl bg-amber-50 flex items-center justify-center">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-amber-500" viewBox="0 0 20 20" fill="currentColor">
                                                <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd"/>
                                            </svg>
                                        </div>
                                        <div>
                                            <p class="font-medium text-stone-900">{desc}</p>
                                            <p class="text-xs text-stone-400 mt-0.5">{format!("Due {}", due)}</p>
                                        </div>
                                    </div>
                                    <div class="flex items-center gap-5">
                                        <p class="text-lg font-semibold text-stone-900 tabular-nums">{format!("${:.2}", amount)}</p>
                                        <button class="btn-primary text-sm">
                                            "Pay Now"
                                        </button>
                                    </div>
                                </div>
                            }
                        }).collect_view()}
                    </div>
                </div>
            </Show>

            // Payment history
            <h2 class="section-title mb-4">"Payment History"</h2>
            <Show
                when=move || !loading.get()
                fallback=|| view! {
                    <div class="card p-12 flex flex-col items-center justify-center">
                        <div class="w-8 h-8 border-2 border-stone-200 border-t-amber-500 rounded-full animate-spin mb-4"></div>
                        <p class="text-sm text-stone-400">"Loading payments\u{2026}"</p>
                    </div>
                }
            >
                <Show
                    when=move || !payments.get().is_empty()
                    fallback=|| view! {
                        <div class="card p-16 flex flex-col items-center justify-center text-center">
                            <div class="w-16 h-16 rounded-2xl bg-stone-100/80 flex items-center justify-center mb-5">
                                <svg xmlns="http://www.w3.org/2000/svg" class="w-8 h-8 text-stone-300" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="1.5">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 14l6-6m-5.5.5h.01m4.99 5h.01M19 21V5a2 2 0 00-2-2H7a2 2 0 00-2 2v16l3.5-2 3.5 2 3.5-2 3.5 2z"/>
                                </svg>
                            </div>
                            <p class="text-stone-900 font-medium mb-1">"No payments yet"</p>
                            <p class="text-sm text-stone-400 max-w-xs">"Your payment history will appear here once you make your first payment."</p>
                        </div>
                    }
                >
                    <div class="card overflow-hidden">
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="border-b border-stone-200/60">
                                    <th class="text-left px-5 py-3.5 stat-label">"Date"</th>
                                    <th class="text-left px-5 py-3.5 stat-label">"Type"</th>
                                    <th class="text-left px-5 py-3.5 stat-label">"Description"</th>
                                    <th class="text-right px-5 py-3.5 stat-label">"Amount"</th>
                                    <th class="text-right px-5 py-3.5 stat-label">"Status"</th>
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
                                        "completed" => "badge badge-success",
                                        "pending" => "badge badge-warning",
                                        _ => "badge badge-error",
                                    };
                                    view! {
                                        <tr class="border-b border-stone-200/60 last:border-0 hover:bg-stone-50/50 transition-colors duration-150">
                                            <td class="px-5 py-4 text-stone-600 tabular-nums">{date}</td>
                                            <td class="px-5 py-4 text-stone-600 capitalize">{ptype}</td>
                                            <td class="px-5 py-4 text-stone-700 font-medium">{desc}</td>
                                            <td class="px-5 py-4 text-right font-semibold text-stone-900 tabular-nums">{format!("${:.2}", amount)}</td>
                                            <td class="px-5 py-4 text-right">
                                                <span class={format!("{} capitalize", badge_class)}>
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

            // Legal notice
            <div class="mt-10 bg-stone-100/50 rounded-2xl p-6 text-sm text-stone-500 leading-relaxed">
                <p class="font-medium text-stone-600 mb-2">"Payment Information"</p>
                <ul class="list-disc list-inside space-y-1.5">
                    <li>"Rent is due on the 1st of each month"</li>
                    <li>"A 5-day grace period applies to electronic payments per WA state law"</li>
                    <li>"Bank transfer (ACH) payments are preferred and have lower processing fees"</li>
                    <li>"You may also pay by check or money order \u{2014} contact your landlord for details"</li>
                </ul>
            </div>
        </div>
    }
}
