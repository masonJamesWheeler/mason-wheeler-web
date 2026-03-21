#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::{PaginatedResponse, Payment};

#[allow(unused)]
#[component]
pub fn AdminPayments() -> impl IntoView {
    let (payments, set_payments) = signal::<Vec<Payment>>(vec![]);
    let (loading, set_loading) = signal(true);
    let (show_utility_form, set_show_utility_form) = signal(false);
    let (util_desc, set_util_desc) = signal(String::new());
    let (util_amount, set_util_amount) = signal(String::new());
    let (util_due, set_util_due) = signal(String::new());
    let (submitting, set_submitting) = signal(false);
    let (search, set_search) = signal(String::new());
    let (search_input, set_search_input) = signal(String::new());
    let (status_filter, set_status_filter) = signal(String::from("all"));
    let (current_page, set_current_page) = signal(1u64);
    let (total_items, set_total_items) = signal(0u64);
    let per_page = 20u64;

    let fetch_payments = move || {
        #[cfg(feature = "hydrate")]
        {
            let search_val = search.get();
            let status_val = status_filter.get();
            let page_val = current_page.get();
            set_loading.set(true);
            leptos::task::spawn_local(async move {
                let mut url = format!("/api/admin/payments?page={}&per_page={}", page_val, per_page);
                if !search_val.is_empty() {
                    url.push_str(&format!("&search={}", search_val.replace('%', "%25").replace('&', "%26").replace('=', "%3D").replace(' ', "%20").replace('+', "%2B")));
                }
                if status_val != "all" {
                    url.push_str(&format!("&status={}", status_val));
                }
                if let Ok(data) = crate::api::client::api_get::<PaginatedResponse<Payment>>(&url).await {
                    set_total_items.set(data.total);
                    set_payments.set(data.items);
                }
                set_loading.set(false);
            });
        }
    };

    // Refetch when search, filter, or page changes
    Effect::new(move |_| {
        let _ = search.get();
        let _ = status_filter.get();
        let _ = current_page.get();
        fetch_payments();
    });

    // Debounced search
    let on_search_input = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        set_search_input.set(val.clone());
        set_current_page.set(1);
        #[cfg(feature = "hydrate")]
        {
            let val = val.clone();
            leptos::task::spawn_local(async move {
                gloo_timers::future::TimeoutFuture::new(300).await;
                if search_input.get() == val {
                    set_search.set(val);
                }
            });
        }
    };

    let on_status_change = move |ev: leptos::ev::Event| {
        let val = event_target_value(&ev);
        set_status_filter.set(val);
        set_current_page.set(1);
    };

    let total_pages = move || {
        let t = total_items.get();
        if t == 0 { 1 } else { (t + per_page - 1) / per_page }
    };

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

                if crate::api::client::api_post_no_body("/api/admin/utilities", &body).await.is_ok() {
                    set_show_utility_form.set(false);
                    set_util_desc.set(String::new());
                    set_util_amount.set(String::new());
                    set_util_due.set(String::new());
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

            // Search and filter controls
            <div class="flex flex-col md:flex-row gap-3 mb-4">
                <input
                    type="text"
                    class="input w-full md:w-80"
                    placeholder="Search by description..."
                    on:input=on_search_input
                    prop:value=move || search_input.get()
                />
                <select
                    class="input w-full md:w-48"
                    on:change=on_status_change
                    prop:value=move || status_filter.get()
                >
                    <option value="all">"All Statuses"</option>
                    <option value="pending">"Pending"</option>
                    <option value="completed">"Completed"</option>
                    <option value="failed">"Failed"</option>
                </select>
            </div>

            <Show
                when=move || !loading.get()
                fallback=|| view! {
                    <div class="card text-center py-12">
                        <div class="w-8 h-8 border-2 border-stone-200 border-t-amber-500 rounded-full animate-spin mx-auto mb-4"></div>
                        <p class="text-stone-400 text-sm">"Loading payments..."</p>
                    </div>
                }
            >
                <Show
                    when=move || !payments.get().is_empty()
                    fallback=|| view! {
                        <div class="card text-center py-12">
                            <p class="text-stone-400 text-sm">"No payments found."</p>
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
                                    let ptype = p.payment_type.to_string();
                                    let desc = p.description.clone().unwrap_or_default();
                                    let amount = p.amount;
                                    let status = p.status.clone();
                                    let status_str = status.to_string();
                                    let badge_class = match status {
                                        mason_wheeler_shared::PaymentStatus::Completed => "badge-success",
                                        mason_wheeler_shared::PaymentStatus::Pending => "badge-warning",
                                        mason_wheeler_shared::PaymentStatus::Failed => "badge-error",
                                    };
                                    view! {
                                        <tr class="border-b border-stone-200/60 last:border-0 hover:bg-stone-50/50 transition-colors">
                                            <td class="px-5 py-3.5 text-stone-600">{date}</td>
                                            <td class="px-5 py-3.5 text-stone-600 capitalize">{ptype}</td>
                                            <td class="px-5 py-3.5 text-stone-700">{desc}</td>
                                            <td class="px-5 py-3.5 text-right font-semibold text-stone-900">{format!("${:.2}", amount)}</td>
                                            <td class="px-5 py-3.5 text-right">
                                                <span class={format!("capitalize {}", badge_class)}>
                                                    {status_str}
                                                </span>
                                            </td>
                                        </tr>
                                    }
                                }).collect_view()}
                            </tbody>
                        </table>
                    </div>

                    // Pagination controls
                    <div class="flex items-center justify-between mt-4">
                        <p class="text-sm text-stone-400">
                            {move || format!("Showing page {} of {}", current_page.get(), total_pages())}
                        </p>
                        <div class="flex gap-2">
                            <button
                                class="btn-secondary text-sm"
                                disabled=move || current_page.get() <= 1
                                on:click=move |_| set_current_page.update(|p| *p = (*p).saturating_sub(1).max(1))
                            >
                                "Previous"
                            </button>
                            <button
                                class="btn-secondary text-sm"
                                disabled=move || current_page.get() >= total_pages()
                                on:click=move |_| set_current_page.update(|p| *p += 1)
                            >
                                "Next"
                            </button>
                        </div>
                    </div>
                </Show>
            </Show>
        </div>
    }
}
