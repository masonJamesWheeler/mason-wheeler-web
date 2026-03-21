#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::{MaintenanceStats, OverviewReport, RevenueMonth};

#[component]
pub fn AdminReports() -> impl IntoView {
    let (data, set_data) = signal::<Option<OverviewReport>>(None);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/admin/reports/overview").send().await {
                if let Ok(d) = resp.json::<OverviewReport>().await {
                    set_data.set(Some(d));
                }
            }
        });
    }

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10">
            <h1 class="text-3xl font-bold tracking-tight text-stone-900 mb-1">"Reports & Analytics"</h1>
            <p class="text-stone-500 mb-8">"Financial overview and property statistics"</p>

            // Overview cards
            <div class="grid grid-cols-1 md:grid-cols-3 gap-5 mb-10">
                <div class="card p-6">
                    <p class="stat-label">"Total Collected"</p>
                    <p class="text-2xl font-bold text-emerald-600 mt-1">
                        {move || data.get().map(|d| format!("${:.2}", d.total_collected)).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-400 mt-1">"All-time revenue"</p>
                </div>
                <div class="card p-6">
                    <p class="stat-label">"Outstanding"</p>
                    <p class="text-2xl font-bold text-amber-500 mt-1">
                        {move || data.get().map(|d| format!("${:.2}", d.total_outstanding)).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-400 mt-1">"Unpaid charges"</p>
                </div>
                <div class="card p-6">
                    <p class="stat-label">"Active Tenants"</p>
                    <p class="text-2xl font-bold text-stone-900 mt-1">
                        {move || data.get().map(|d| d.active_tenants.to_string()).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-400 mt-1">"Current residents"</p>
                </div>
            </div>

            // Maintenance stats
            <h2 class="section-title mb-5">"Maintenance Summary"</h2>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-4 mb-10">
                <div class="card p-5 text-center">
                    <p class="text-2xl font-bold text-stone-900">
                        {move || data.get().map(|d| d.maintenance_stats.total.to_string()).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-500 mt-1">"Total Requests"</p>
                </div>
                <div class="card p-5 text-center">
                    <p class="text-2xl font-bold text-blue-600">
                        {move || data.get().map(|d| d.maintenance_stats.submitted.to_string()).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-500 mt-1">"Submitted"</p>
                </div>
                <div class="card p-5 text-center">
                    <p class="text-2xl font-bold text-amber-500">
                        {move || data.get().map(|d| d.maintenance_stats.in_progress.to_string()).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-500 mt-1">"In Progress"</p>
                </div>
                <div class="card p-5 text-center">
                    <p class="text-2xl font-bold text-emerald-600">
                        {move || data.get().map(|d| d.maintenance_stats.completed.to_string()).unwrap_or_else(|| "...".to_string())}
                    </p>
                    <p class="text-xs text-stone-500 mt-1">"Completed"</p>
                </div>
            </div>

            // Monthly revenue table
            <h2 class="section-title mb-5">"Monthly Revenue (Last 12 Months)"</h2>
            <div class="card overflow-hidden">
                {move || match data.get() {
                    Some(d) if !d.monthly_revenue.is_empty() => view! {
                        <table class="w-full text-sm">
                            <thead>
                                <tr class="border-b border-stone-200 bg-stone-50">
                                    <th class="text-left px-5 py-3 font-medium text-stone-500">"Month"</th>
                                    <th class="text-right px-5 py-3 font-medium text-stone-500">"Rent"</th>
                                    <th class="text-right px-5 py-3 font-medium text-stone-500">"Utility"</th>
                                    <th class="text-right px-5 py-3 font-medium text-stone-500">"Total"</th>
                                    <th class="text-right px-5 py-3 font-medium text-stone-500">"Payments"</th>
                                </tr>
                            </thead>
                            <tbody>
                                {d.monthly_revenue.into_iter().map(|m| view! {
                                    <tr class="border-b border-stone-100 last:border-0">
                                        <td class="px-5 py-3 font-medium text-stone-900">{m.month}</td>
                                        <td class="px-5 py-3 text-right text-stone-700">{format!("${:.2}", m.rent)}</td>
                                        <td class="px-5 py-3 text-right text-stone-700">{format!("${:.2}", m.utility)}</td>
                                        <td class="px-5 py-3 text-right font-semibold text-stone-900">{format!("${:.2}", m.total)}</td>
                                        <td class="px-5 py-3 text-right text-stone-500">{m.count}</td>
                                    </tr>
                                }).collect::<Vec<_>>()}
                            </tbody>
                        </table>
                    }.into_any(),
                    Some(_) => view! {
                        <div class="px-5 py-8 text-center text-stone-400">"No revenue data available."</div>
                    }.into_any(),
                    None => view! {
                        <div class="px-5 py-8 text-center text-stone-400">"Loading..."</div>
                    }.into_any(),
                }}
            </div>
        </div>
    }
}
