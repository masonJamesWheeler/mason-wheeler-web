use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Header() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);

    // Check current path to conditionally hide sign-in on login page
    #[cfg(feature = "hydrate")]
    let is_login = {
        let (val, set_val) = signal(false);
        leptos::task::spawn_local(async move {
            if let Some(window) = web_sys::window() {
                if let Ok(path) = window.location().pathname() {
                    set_val.set(path == "/login");
                }
            }
        });
        val
    };
    #[cfg(not(feature = "hydrate"))]
    let is_login = signal(false).0;

    view! {
        <header class="sticky top-0 z-50 backdrop-blur-xl bg-white/80 border-b border-stone-200/60">
            <div class="max-w-5xl mx-auto px-6 h-16 flex items-center justify-between">
                <A href="/" attr:class="flex items-center gap-3 group">
                    <div class="w-8 h-8 rounded-lg bg-stone-800 flex items-center justify-center">
                        <svg class="w-4 h-4 text-white" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                        </svg>
                    </div>
                    <span class="hidden sm:block text-sm font-semibold text-stone-900 leading-none">"8404 12th Ave S"</span>
                </A>

                // Desktop nav — hide sign-in on login page
                <nav class="hidden md:flex items-center gap-1">
                    <Show when=move || !is_login.get()>
                        <A href="/login" attr:class="btn-primary text-sm px-4 py-2">
                            "Sign in"
                        </A>
                    </Show>
                </nav>

                // Mobile toggle
                <button
                    class="md:hidden w-10 h-10 flex items-center justify-center rounded-xl hover:bg-stone-100 transition-colors"
                    on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                >
                    <svg class="w-5 h-5 text-stone-600" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
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

            // Mobile nav
            <Show when=move || mobile_open.get()>
                <nav class="md:hidden border-t border-stone-100 px-6 py-4 bg-white space-y-1">
                    <Show when=move || !is_login.get()>
                        <A href="/login" attr:class="block py-2.5 px-3 rounded-xl text-sm font-medium text-stone-700 hover:bg-stone-50 transition-colors">"Sign in"</A>
                    </Show>
                </nav>
            </Show>
        </header>
    }
}
