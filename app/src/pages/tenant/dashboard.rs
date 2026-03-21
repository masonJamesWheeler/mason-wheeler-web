#![allow(unused)]
use leptos::prelude::*;
use leptos_router::components::A;

#[allow(unused)]
#[component]
pub fn TenantDashboard() -> impl IntoView {
    let (balance, set_balance) = signal(3250.0_f64);
    let (next_due, set_next_due) = signal("April 1, 2026".to_string());
    let (maintenance_count, set_maintenance_count) = signal(0_u32);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(data) = crate::api::client::api_get::<mason_wheeler_shared::DashboardData>("/api/tenant/dashboard").await {
                set_balance.set(data.amount_due);
                if let Some(date) = data.next_due_date {
                    set_next_due.set(date);
                }
                set_maintenance_count.set(data.active_maintenance.len() as u32);
            }
        });
    }

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10">
            // Greeting
            <div class="mb-10">
                <p class="stat-label mb-1">"Overview"</p>
                <h1 class="text-2xl font-semibold text-stone-900 tracking-tight">"Your residence"</h1>
            </div>

            // Stats row
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4 mb-10">
                // Amount due — primary stat
                <div class="card p-6 sm:col-span-1 relative overflow-hidden">
                    <div class="absolute top-0 right-0 w-24 h-24 bg-amber-500/5 rounded-full -translate-y-8 translate-x-8" />
                    <p class="stat-label">"Amount due"</p>
                    <p class="text-3xl font-semibold text-stone-900 mt-1.5 tracking-tight tabular-nums">
                        {move || format!("${:.2}", balance.get())}
                    </p>
                    <p class="text-xs text-stone-400 mt-2">
                        {move || format!("Due {}", next_due.get())}
                    </p>
                </div>

                // Lease status
                <div class="card p-6">
                    <p class="stat-label">"Lease"</p>
                    <div class="flex items-center gap-2 mt-2">
                        <span class="w-2 h-2 rounded-full bg-emerald-500 animate-pulse" />
                        <span class="text-lg font-semibold text-stone-900">"Active"</span>
                    </div>
                    <p class="text-xs text-stone-400 mt-2">"12-month term"</p>
                </div>

                // Maintenance
                <div class="card p-6">
                    <p class="stat-label">"Open requests"</p>
                    <p class="text-3xl font-semibold text-stone-900 mt-1.5 tracking-tight tabular-nums">
                        {move || maintenance_count.get()}
                    </p>
                    <p class="text-xs text-stone-400 mt-2">"Maintenance"</p>
                </div>
            </div>

            // Primary CTA
            <A href="/tenant/payments"
                attr:class="flex items-center justify-between card card-hover p-6 mb-4 group cursor-pointer"
            >
                <div class="flex items-center gap-5">
                    <div class="w-12 h-12 rounded-xl bg-amber-500 flex items-center justify-center shadow-lg shadow-amber-500/20">
                        <svg class="w-6 h-6 text-white" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 18.75a60.07 60.07 0 0115.797 2.101c.727.198 1.453-.342 1.453-1.096V18.75M3.75 4.5v.75A.75.75 0 013 6h-.75m0 0v-.375c0-.621.504-1.125 1.125-1.125H20.25M2.25 6v9m18-10.5v.75c0 .414.336.75.75.75h.75m-1.5-1.5h.375c.621 0 1.125.504 1.125 1.125v9.75c0 .621-.504 1.125-1.125 1.125h-.375m1.5-1.5H21a.75.75 0 00-.75.75v.75m0 0H3.75m0 0h-.375a1.125 1.125 0 01-1.125-1.125V15m1.5 1.5v-.75A.75.75 0 003 15h-.75M15 10.5a3 3 0 11-6 0 3 3 0 016 0zm3 0h.008v.008H18V10.5zm-12 0h.008v.008H6V10.5z" />
                        </svg>
                    </div>
                    <div>
                        <p class="text-[15px] font-semibold text-stone-900">"Pay rent"</p>
                        <p class="text-sm text-stone-400">"Bank transfer or card"</p>
                    </div>
                </div>
                <svg class="w-5 h-5 text-stone-300 group-hover:text-stone-500 group-hover:translate-x-0.5 transition-all" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                </svg>
            </A>

            // Secondary actions
            <div class="grid grid-cols-1 sm:grid-cols-3 gap-4">
                <A href="/tenant/maintenance"
                    attr:class="card card-hover p-5 group cursor-pointer"
                >
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-stone-100 flex items-center justify-center group-hover:bg-stone-200 transition-colors">
                                <svg class="w-5 h-5 text-stone-500" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M11.42 15.17l-5.645 3.025a.563.563 0 01-.756-.395C4.71 16.16 5.22 10.65 8.075 7.41a6.46 6.46 0 015.08-2.41m2.08 5.75c-.53 2.02-2.26 3.77-4.44 4.42M17.21 7.68l.07-.06c.87-.87 2.36-.75 3.07.28.75 1.07.48 2.51-.55 3.28l-.54.38M11.42 15.17l2.496-4.328" />
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm font-medium text-stone-900">"Maintenance"</p>
                                <p class="text-xs text-stone-400">"Request repairs"</p>
                            </div>
                        </div>
                        <svg class="w-4 h-4 text-stone-300" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                        </svg>
                    </div>
                </A>

                <A href="/tenant/documents"
                    attr:class="card card-hover p-5 group cursor-pointer"
                >
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-stone-100 flex items-center justify-center group-hover:bg-stone-200 transition-colors">
                                <svg class="w-5 h-5 text-stone-500" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm font-medium text-stone-900">"Documents"</p>
                                <p class="text-xs text-stone-400">"Lease & disclosures"</p>
                            </div>
                        </div>
                        <svg class="w-4 h-4 text-stone-300" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                        </svg>
                    </div>
                </A>

                <A href="/tenant/payments"
                    attr:class="card card-hover p-5 group cursor-pointer"
                >
                    <div class="flex items-center justify-between">
                        <div class="flex items-center gap-4">
                            <div class="w-10 h-10 rounded-xl bg-stone-100 flex items-center justify-center group-hover:bg-stone-200 transition-colors">
                                <svg class="w-5 h-5 text-stone-500" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 6v6h4.5m4.5 0a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <div>
                                <p class="text-sm font-medium text-stone-900">"History"</p>
                                <p class="text-xs text-stone-400">"Past payments"</p>
                            </div>
                        </div>
                        <svg class="w-4 h-4 text-stone-300" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M8.25 4.5l7.5 7.5-7.5 7.5" />
                        </svg>
                    </div>
                </A>
            </div>

            // Legal notice
            <div class="mt-10 px-5 py-4 rounded-xl bg-stone-100/50 border border-stone-200/40">
                <p class="text-xs text-stone-400 leading-relaxed">
                    "Rent is due on the 1st of each month. A 5-day grace period applies to electronic payments per Washington State law (RCW 59.18). You may also pay by check or money order."
                </p>
            </div>
        </div>
    }
}
