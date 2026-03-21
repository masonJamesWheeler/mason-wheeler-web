#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::TenantUser;

#[allow(unused)]
#[component]
pub fn AdminTenants() -> impl IntoView {
    let (tenants, set_tenants) = signal::<Vec<TenantUser>>(vec![]);
    let (show_form, set_show_form) = signal(false);
    let (tenant_name, set_tenant_name) = signal(String::new());
    let (tenant_email, set_tenant_email) = signal(String::new());
    let (tenant_password, set_tenant_password) = signal(String::new());
    let (submitting, set_submitting) = signal(false);

    #[cfg(feature = "hydrate")]
    {
        let set_tenants = set_tenants.clone();
        leptos::task::spawn_local(async move {
            if let Ok(data) = crate::api::client::api_get::<Vec<TenantUser>>("/api/admin/tenants").await {
                set_tenants.set(data);
            }
        });
    }

    let handle_add_tenant = move |ev: leptos::ev::SubmitEvent| {
        ev.prevent_default();
        set_submitting.set(true);

        #[cfg(feature = "hydrate")]
        {
            let name = tenant_name.get();
            let email = tenant_email.get();
            let password = tenant_password.get();

            leptos::task::spawn_local(async move {
                let body = serde_json::json!({
                    "name": name,
                    "email": email,
                    "password": password,
                });

                if let Ok(tenant) = crate::api::client::api_post::<TenantUser>("/api/admin/tenants", &body).await {
                    set_tenants.update(|t| t.insert(0, tenant));
                    set_show_form.set(false);
                    set_tenant_name.set(String::new());
                    set_tenant_email.set(String::new());
                    set_tenant_password.set(String::new());
                }
                set_submitting.set(false);
            });
        }
    };

    let handle_delete = move |id: String| {
        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async move {
                let url = format!("/api/admin/tenants/{}", id);
                if crate::api::client::api_delete(&url).await.is_ok() {
                    set_tenants.update(|t| t.retain(|tenant| tenant.id != id));
                }
            });
        }
    };

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10">
            <div class="flex items-center justify-between mb-8">
                <div>
                    <h1 class="text-3xl font-bold tracking-tight text-stone-900">"Manage Tenants"</h1>
                    <p class="text-stone-500 mt-1">"Add, view, and remove tenant accounts"</p>
                </div>
                <button
                    class="btn-primary"
                    on:click=move |_| set_show_form.update(|v| *v = !*v)
                >
                    {move || if show_form.get() { "Cancel" } else { "Add Tenant" }}
                </button>
            </div>

            // Add tenant form
            <Show when=move || show_form.get()>
                <form on:submit=handle_add_tenant class="card mb-8">
                    <h2 class="section-title">"Add Tenant"</h2>
                    <div class="grid grid-cols-1 md:grid-cols-3 gap-5">
                        <div>
                            <label class="block text-sm font-medium text-stone-600 mb-1.5">"Name"</label>
                            <input
                                type="text"
                                required=true
                                class="input"
                                placeholder="Full name"
                                on:input=move |ev| set_tenant_name.set(event_target_value(&ev))
                                prop:value=move || tenant_name.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-stone-600 mb-1.5">"Email"</label>
                            <input
                                type="email"
                                required=true
                                class="input"
                                placeholder="tenant@example.com"
                                on:input=move |ev| set_tenant_email.set(event_target_value(&ev))
                                prop:value=move || tenant_email.get()
                            />
                        </div>
                        <div>
                            <label class="block text-sm font-medium text-stone-600 mb-1.5">"Password"</label>
                            <input
                                type="password"
                                required=true
                                class="input"
                                placeholder="Initial password"
                                on:input=move |ev| set_tenant_password.set(event_target_value(&ev))
                                prop:value=move || tenant_password.get()
                            />
                        </div>
                    </div>
                    <div class="mt-6 flex justify-end">
                        <button
                            type="submit"
                            class="btn-accent"
                            disabled=move || submitting.get()
                        >
                            {move || if submitting.get() { "Adding..." } else { "Add Tenant" }}
                        </button>
                    </div>
                </form>
            </Show>

            // Tenant list
            <h2 class="section-title">"All Tenants"</h2>
            <Show
                when=move || !tenants.get().is_empty()
                fallback=|| view! {
                    <div class="card text-center py-12">
                        <p class="text-stone-400 text-sm">"No tenants added yet."</p>
                    </div>
                }
            >
                <div class="card !p-0 overflow-hidden">
                    <table class="w-full text-sm">
                        <thead>
                            <tr class="border-b border-stone-200/60">
                                <th class="text-left px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Name"</th>
                                <th class="text-left px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Email"</th>
                                <th class="text-left px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Created"</th>
                                <th class="text-right px-5 py-3.5 text-xs font-semibold uppercase tracking-wider text-stone-400">"Actions"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {move || tenants.get().iter().map(|t| {
                                let id = t.id.clone();
                                let name = t.name.clone();
                                let email = t.email.clone();
                                let created = t.created_at.clone();
                                let delete_id = id.clone();
                                view! {
                                    <tr class="border-b border-stone-200/60 last:border-0 hover:bg-stone-50/50 transition-colors">
                                        <td class="px-5 py-3.5 text-stone-700 font-medium">{name}</td>
                                        <td class="px-5 py-3.5 text-stone-600">{email}</td>
                                        <td class="px-5 py-3.5 text-stone-600">{created}</td>
                                        <td class="px-5 py-3.5 text-right">
                                            <button
                                                class="text-sm text-red-500 hover:text-red-700 font-medium transition-colors"
                                                on:click=move |_| {
                                                    let id = delete_id.clone();
                                                    #[cfg(feature = "hydrate")]
                                                    {
                                                        if let Some(window) = web_sys::window() {
                                                            if window.confirm_with_message("Are you sure you want to delete this tenant?").unwrap_or(false) {
                                                                handle_delete(id);
                                                            }
                                                        }
                                                    }
                                                }
                                            >
                                                "Delete"
                                            </button>
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
