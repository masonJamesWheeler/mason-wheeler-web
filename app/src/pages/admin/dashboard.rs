use leptos::prelude::*;
use leptos_router::components::A;
use mason_wheeler_shared::DashboardData;

#[component]
pub fn AdminDashboard() -> impl IntoView {
    let (data, set_data) = signal::<Option<DashboardData>>(None);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/admin/dashboard").send().await {
                if let Ok(d) = resp.json::<DashboardData>().await {
                    set_data.set(Some(d));
                }
            }
        });
    }

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-slate-900 mb-6">"Admin Dashboard"</h1>

            <div class="grid grid-cols-1 md:grid-cols-3 gap-4 mb-8">
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    <p class="text-sm text-slate-500">"Rent Status"</p>
                    {move || match data.get() {
                        Some(d) if d.amount_due > 0.0 => view! {
                            <p class="text-2xl font-bold text-yellow-600">{format!("${:.2} due", d.amount_due)}</p>
                        }.into_any(),
                        Some(_) => view! {
                            <p class="text-2xl font-bold text-green-600">"Paid"</p>
                        }.into_any(),
                        None => view! {
                            <p class="text-xl text-slate-400">"Loading..."</p>
                        }.into_any(),
                    }}
                </div>
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    <p class="text-sm text-slate-500">"Active Maintenance"</p>
                    <p class="text-2xl font-bold text-slate-900">
                        {move || data.get().map(|d| d.active_maintenance.len()).unwrap_or(0)}
                    </p>
                </div>
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    <p class="text-sm text-slate-500">"Outstanding Utilities"</p>
                    <p class="text-2xl font-bold text-slate-900">
                        {move || {
                            data.get()
                                .map(|d| {
                                    let total: f64 = d.utility_charges.iter().map(|u| u.amount).sum();
                                    format!("${:.2}", total)
                                })
                                .unwrap_or_else(|| "...".to_string())
                        }}
                    </p>
                </div>
            </div>

            <h2 class="text-lg font-semibold text-slate-900 mb-4">"Quick Actions"</h2>
            <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
                <A href="/admin/payments"
                    attr:class="flex items-center gap-4 bg-white border border-slate-200 hover:border-slate-300 rounded-xl p-6 transition-colors"
                >
                    <div>
                        <p class="font-semibold text-lg text-slate-900">"Manage Payments"</p>
                        <p class="text-slate-500 text-sm">"View history, add utility charges"</p>
                    </div>
                </A>
                <A href="/admin/maintenance"
                    attr:class="flex items-center gap-4 bg-white border border-slate-200 hover:border-slate-300 rounded-xl p-6 transition-colors"
                >
                    <div>
                        <p class="font-semibold text-lg text-slate-900">"Maintenance Requests"</p>
                        <p class="text-slate-500 text-sm">"View and respond to tenant requests"</p>
                    </div>
                </A>
                <A href="/admin/documents"
                    attr:class="flex items-center gap-4 bg-white border border-slate-200 hover:border-slate-300 rounded-xl p-6 transition-colors"
                >
                    <div>
                        <p class="font-semibold text-lg text-slate-900">"Documents"</p>
                        <p class="text-slate-500 text-sm">"Upload lease, disclosures, receipts"</p>
                    </div>
                </A>
                <A href="/admin/payments"
                    attr:class="flex items-center gap-4 bg-orange-600 hover:bg-orange-700 text-white rounded-xl p-6 transition-colors"
                >
                    <div>
                        <p class="font-semibold text-lg">"Add Utility Charge"</p>
                        <p class="text-orange-100 text-sm">"Pass through utility costs to tenant"</p>
                    </div>
                </A>
            </div>
        </div>
    }
}
