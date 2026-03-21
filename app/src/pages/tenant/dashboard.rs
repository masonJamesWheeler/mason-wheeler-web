#![allow(unused)]
use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn TenantDashboard() -> impl IntoView {
    let (balance, set_balance) = signal(3250.0_f64);
    let (next_due, set_next_due) = signal("April 1, 2026".to_string());
    let (maintenance_count, set_maintenance_count) = signal(0_u32);

    // Fetch dashboard data on mount
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/tenant/dashboard")
                .send()
                .await
            {
                if let Ok(data) = resp.json::<mason_wheeler_shared::DashboardData>().await {
                    set_balance.set(data.amount_due);
                    if let Some(date) = data.next_due_date {
                        set_next_due.set(date);
                    }
                    set_maintenance_count.set(data.active_maintenance.len() as u32);
                }
            }
        });
    }

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-slate-900 mb-6">"Dashboard"</h1>

            // Quick stats
            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    <p class="text-sm text-slate-500">"Amount Due"</p>
                    <p class="text-3xl font-bold text-slate-900">{move || format!("${:.2}", balance.get())}</p>
                    <p class="text-sm text-slate-500 mt-1">{move || format!("Due {}", next_due.get())}</p>
                </div>
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    <p class="text-sm text-slate-500">"Lease Status"</p>
                    <p class="text-xl font-bold text-green-600">"Active"</p>
                    <p class="text-sm text-slate-500 mt-1">"12-month term"</p>
                </div>
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    <p class="text-sm text-slate-500">"Open Requests"</p>
                    <p class="text-3xl font-bold text-slate-900">{move || maintenance_count.get()}</p>
                    <p class="text-sm text-slate-500 mt-1">"Maintenance"</p>
                </div>
            </div>

            // Quick actions
            <h2 class="text-lg font-semibold text-slate-900 mb-4">"Quick Actions"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <A href="/tenant/payments"
                    attr:class="flex items-center gap-4 bg-orange-600 hover:bg-orange-700 text-white rounded-xl p-6 transition-colors"
                >
                    <svg class="w-8 h-8" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M17 9V7a2 2 0 00-2-2H5a2 2 0 00-2 2v6a2 2 0 002 2h2m2 4h10a2 2 0 002-2v-6a2 2 0 00-2-2H9a2 2 0 00-2 2v6a2 2 0 002 2zm7-5a2 2 0 11-4 0 2 2 0 014 0z" />
                    </svg>
                    <div>
                        <p class="font-semibold text-lg">"Pay Rent"</p>
                        <p class="text-orange-100 text-sm">"Make a payment via bank transfer or card"</p>
                    </div>
                </A>

                <A href="/tenant/maintenance"
                    attr:class="flex items-center gap-4 bg-white border border-slate-200 hover:border-slate-300 rounded-xl p-6 transition-colors"
                >
                    <svg class="w-8 h-8 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M10.325 4.317c.426-1.756 2.924-1.756 3.35 0a1.724 1.724 0 002.573 1.066c1.543-.94 3.31.826 2.37 2.37a1.724 1.724 0 001.066 2.573c1.756.426 1.756 2.924 0 3.35a1.724 1.724 0 00-1.066 2.573c.94 1.543-.826 3.31-2.37 2.37a1.724 1.724 0 00-2.573 1.066c-.426 1.756-2.924 1.756-3.35 0a1.724 1.724 0 00-2.573-1.066c-1.543.94-3.31-.826-2.37-2.37a1.724 1.724 0 00-1.066-2.573c-1.756-.426-1.756-2.924 0-3.35a1.724 1.724 0 001.066-2.573c-.94-1.543.826-3.31 2.37-2.37.996.608 2.296.07 2.572-1.065z" />
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                    </svg>
                    <div>
                        <p class="font-semibold text-lg text-slate-900">"Maintenance Request"</p>
                        <p class="text-slate-500 text-sm">"Report an issue or request repairs"</p>
                    </div>
                </A>

                <A href="/tenant/documents"
                    attr:class="flex items-center gap-4 bg-white border border-slate-200 hover:border-slate-300 rounded-xl p-6 transition-colors"
                >
                    <svg class="w-8 h-8 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                    </svg>
                    <div>
                        <p class="font-semibold text-lg text-slate-900">"Documents"</p>
                        <p class="text-slate-500 text-sm">"View lease, disclosures, and receipts"</p>
                    </div>
                </A>

                <A href="/tenant/payments"
                    attr:class="flex items-center gap-4 bg-white border border-slate-200 hover:border-slate-300 rounded-xl p-6 transition-colors"
                >
                    <svg class="w-8 h-8 text-slate-600" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8c-1.657 0-3 .895-3 2s1.343 2 3 2 3 .895 3 2-1.343 2-3 2m0-8c1.11 0 2.08.402 2.599 1M12 8V7m0 1v8m0 0v1m0-1c-1.11 0-2.08-.402-2.599-1M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <div>
                        <p class="font-semibold text-lg text-slate-900">"Payment History"</p>
                        <p class="text-slate-500 text-sm">"View past payments and receipts"</p>
                    </div>
                </A>
            </div>
        </div>
    }
}
