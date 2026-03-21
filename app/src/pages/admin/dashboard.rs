#![allow(unused)]
use leptos::prelude::*;
use leptos_router::components::A;
use mason_wheeler_shared::DashboardData;

#[allow(unused)]
#[component]
pub fn AdminDashboard() -> impl IntoView {
    let (data, set_data) = signal::<Option<DashboardData>>(None);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(d) = crate::api::client::api_get::<DashboardData>("/api/admin/dashboard").await {
                set_data.set(Some(d));
            }
        });
    }

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10">
            <h1 class="text-3xl font-bold tracking-tight text-stone-900 mb-1">"Admin Dashboard"</h1>
            <p class="text-stone-500 mb-8">"Property overview and quick actions"</p>

            // Stats row
            <div class="grid grid-cols-1 md:grid-cols-3 gap-5 mb-10">
                <div class="card p-6">
                    <p class="stat-label">"Rent Status"</p>
                    {move || match data.get() {
                        Some(d) if d.amount_due > 0.0 => view! {
                            <p class="text-2xl font-bold text-amber-500 mt-1">{format!("${:.2} due", d.amount_due)}</p>
                            <p class="text-xs text-stone-400 mt-1">"Outstanding balance"</p>
                        }.into_any(),
                        Some(_) => view! {
                            <p class="text-2xl font-bold text-emerald-600 mt-1">"Paid"</p>
                            <p class="text-xs text-stone-400 mt-1">"Current period settled"</p>
                        }.into_any(),
                        None => view! {
                            <p class="text-2xl font-bold text-stone-300 mt-1">"..."</p>
                            <p class="text-xs text-stone-400 mt-1">"Loading"</p>
                        }.into_any(),
                    }}
                </div>
                <div class="card p-6">
                    <p class="stat-label">"Active Maintenance"</p>
                    <p class="text-2xl font-bold text-stone-900 mt-1">
                        {move || data.get().map(|d| d.active_maintenance.len()).unwrap_or(0)}
                    </p>
                    <p class="text-xs text-stone-400 mt-1">"Open requests"</p>
                </div>
                <div class="card p-6">
                    <p class="stat-label">"Outstanding Utilities"</p>
                    <p class="text-2xl font-bold text-stone-900 mt-1">
                        {move || {
                            data.get()
                                .map(|d| {
                                    let total: f64 = d.utility_charges.iter().map(|u| u.amount).sum();
                                    format!("${:.2}", total.abs())
                                })
                                .unwrap_or_else(|| "...".to_string())
                        }}
                    </p>
                    <p class="text-xs text-stone-400 mt-1">"Pending charges"</p>
                </div>
            </div>

            // Quick Actions
            <h2 class="section-title mb-5">"Quick Actions"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <A href="/admin/payments"
                    attr:class="card card-hover p-6 group flex items-center justify-between"
                >
                    <div>
                        <p class="font-semibold text-lg text-stone-900">"Manage Payments"</p>
                        <p class="text-stone-500 text-sm mt-0.5">"View history, add utility charges"</p>
                    </div>
                    <svg class="w-5 h-5 text-stone-400 group-hover:text-amber-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                </A>
                <A href="/admin/maintenance"
                    attr:class="card card-hover p-6 group flex items-center justify-between"
                >
                    <div>
                        <p class="font-semibold text-lg text-stone-900">"Maintenance Requests"</p>
                        <p class="text-stone-500 text-sm mt-0.5">"View and respond to tenant requests"</p>
                    </div>
                    <svg class="w-5 h-5 text-stone-400 group-hover:text-amber-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                </A>
                <A href="/admin/documents"
                    attr:class="card card-hover p-6 group flex items-center justify-between"
                >
                    <div>
                        <p class="font-semibold text-lg text-stone-900">"Documents"</p>
                        <p class="text-stone-500 text-sm mt-0.5">"Upload lease, disclosures, receipts"</p>
                    </div>
                    <svg class="w-5 h-5 text-stone-400 group-hover:text-amber-500 transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                </A>
                <A href="/admin/payments"
                    attr:class="card p-6 group flex items-center justify-between bg-amber-500 hover:bg-amber-600 border-amber-500 hover:shadow-lg hover:shadow-amber-500/20 hover:-translate-y-0.5 transition-all"
                >
                    <div>
                        <p class="font-semibold text-lg text-white">"Add Utility Charge"</p>
                        <p class="text-amber-100 text-sm mt-0.5">"Pass through utility costs to tenant"</p>
                    </div>
                    <svg class="w-5 h-5 text-amber-200 group-hover:text-white transition-colors" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                    </svg>
                </A>
            </div>
        </div>
    }
}
