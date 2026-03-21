use leptos::prelude::*;
use leptos_router::components::A;

use crate::state::auth::AuthState;

/// Auth-aware header. If AuthState is in context (provided by AuthenticatedLayout),
/// shows nav links + user info. Otherwise shows minimal public header.
#[component]
pub fn Header() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);

    // Try to get auth context — None if we're on a public page
    let auth = leptos::context::use_context::<AuthState>();

    let is_logged_in = move || {
        auth.as_ref()
            .and_then(|a| a.user.get())
            .is_some()
    };

    let is_landlord = move || {
        auth.as_ref()
            .and_then(|a| a.user.get())
            .map(|u| u.role == "landlord")
            .unwrap_or(false)
    };

    let user_name = Signal::derive(move || {
        auth.as_ref()
            .and_then(|a| a.user.get())
            .map(|u| u.name.clone())
            .unwrap_or_default()
    });

    let base_path = Signal::derive(move || {
        if is_landlord() { "/admin".to_string() } else { "/tenant".to_string() }
    });

    let handle_logout = move |_| {
        #[cfg(feature = "hydrate")]
        {
            leptos::task::spawn_local(async {
                let _ = crate::api::client::api_post_no_body(
                    "/api/auth/logout",
                    &serde_json::json!({}),
                ).await;
                if let Some(window) = web_sys::window() {
                    let _ = window.location().set_href("/");
                }
            });
        }
    };

    // Consistent nav link style
    let nav_link = "text-sm font-medium text-stone-600 hover:text-stone-900 px-3 py-2 rounded-lg hover:bg-stone-100 transition-colors";
    let mobile_link = "block py-2.5 px-3 rounded-lg text-sm font-medium text-stone-700 hover:bg-stone-50 transition-colors";

    view! {
        <header class="sticky top-0 z-50 border-b border-stone-200 bg-white/95 backdrop-blur-sm">
            <div class="mx-auto max-w-5xl px-6">
                <div class="flex h-14 items-center justify-between">
                    // Logo
                    <A href="/" attr:class="flex items-center gap-2.5">
                        <div class="flex h-7 w-7 items-center justify-center rounded-lg bg-stone-800">
                            <svg class="h-3.5 w-3.5 text-white" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                            </svg>
                        </div>
                        <span class="hidden sm:block text-sm font-semibold text-stone-900">"8404 12th Ave S"</span>
                    </A>

                    // Desktop nav
                    <div class="hidden md:flex md:items-center md:space-x-1">
                        <Show when=move || is_logged_in()>
                            <A href={move || base_path.get()} attr:class=nav_link>"Dashboard"</A>
                            <A href={move || format!("{}/payments", base_path.get())} attr:class=nav_link>"Payments"</A>
                            <A href={move || format!("{}/documents", base_path.get())} attr:class=nav_link>"Documents"</A>
                            <A href={move || format!("{}/maintenance", base_path.get())} attr:class=nav_link>"Maintenance"</A>

                            // User dropdown — pure CSS hover
                            <div class="group relative ml-2">
                                <button class="flex h-8 w-8 cursor-pointer items-center justify-center rounded-full bg-stone-700 text-xs font-semibold text-white transition-colors hover:bg-stone-800">
                                    {move || {
                                        let name = user_name.get();
                                        name.split_whitespace()
                                            .filter_map(|w| w.chars().next())
                                            .take(2)
                                            .collect::<String>()
                                    }}
                                </button>
                                <div class="invisible absolute right-0 top-full z-50 mt-1 min-w-[180px] opacity-0 transition-all duration-200 group-hover:visible group-hover:opacity-100">
                                    <div class="overflow-hidden rounded-lg border border-stone-200 bg-white py-1 shadow-lg">
                                        <div class="border-b border-stone-100 px-4 py-2">
                                            <p class="text-sm font-medium text-stone-900">{move || user_name.get()}</p>
                                        </div>
                                        <Show when=move || is_landlord()>
                                            <A href="/admin/reports" attr:class="block px-4 py-2 text-sm text-stone-700 hover:bg-stone-100">"Reports"</A>
                                            <A href="/admin/tenants" attr:class="block px-4 py-2 text-sm text-stone-700 hover:bg-stone-100">"Manage Tenants"</A>
                                        </Show>
                                        <div class="border-t border-stone-100">
                                            <button
                                                class="block w-full px-4 py-2 text-left text-sm text-red-600 hover:bg-red-50 cursor-pointer"
                                                on:click=handle_logout
                                            >"Sign out"</button>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </Show>

                        // Public: show sign-in button
                        <Show when=move || !is_logged_in()>
                            <A href="/login" attr:class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-lg bg-stone-900 text-white hover:bg-stone-800 transition-colors">
                                "Sign in"
                            </A>
                        </Show>
                    </div>

                    // Mobile hamburger
                    <button
                        class="flex md:hidden h-9 w-9 items-center justify-center rounded-lg text-stone-500 hover:bg-stone-100 transition-colors"
                        on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                    >
                        <svg class="h-5 w-5" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <Show
                                when=move || mobile_open.get()
                                fallback=|| view! {
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M3.75 9h16.5m-16.5 6.75h16.5" />
                                }
                            >
                                <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                            </Show>
                        </svg>
                    </button>
                </div>
            </div>

            // Mobile menu
            <div
                class="border-t border-stone-200 bg-white md:hidden"
                style:display=move || if mobile_open.get() { "" } else { "none" }
            >
                <div class="px-4 py-3 space-y-0.5">
                    <Show when=move || is_logged_in()>
                        <A href={move || base_path.get()} attr:class=mobile_link>"Dashboard"</A>
                        <A href={move || format!("{}/payments", base_path.get())} attr:class=mobile_link>"Payments"</A>
                        <A href={move || format!("{}/documents", base_path.get())} attr:class=mobile_link>"Documents"</A>
                        <A href={move || format!("{}/maintenance", base_path.get())} attr:class=mobile_link>"Maintenance"</A>
                        <div class="border-t border-stone-100 my-2" />
                        <div class="px-3 py-2">
                            <p class="text-sm font-medium text-stone-900">{move || user_name.get()}</p>
                        </div>
                        <button
                            class="block w-full text-left py-2.5 px-3 rounded-lg text-sm text-red-600 hover:bg-red-50 transition-colors"
                            on:click=handle_logout
                        >"Sign out"</button>
                    </Show>
                    <Show when=move || !is_logged_in()>
                        <A href="/login" attr:class=mobile_link>"Sign in"</A>
                    </Show>
                </div>
            </div>
        </header>
    }
}
