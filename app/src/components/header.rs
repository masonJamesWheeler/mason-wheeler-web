use leptos::prelude::*;
use leptos_router::components::A;

#[component]
pub fn Header() -> impl IntoView {
    let (mobile_open, set_mobile_open) = signal(false);

    view! {
        <header class="bg-slate-900 text-white shadow-lg">
            <div class="max-w-6xl mx-auto px-4 py-4 flex items-center justify-between">
                <A href="/" attr:class="text-xl font-bold tracking-tight hover:text-orange-400 transition-colors">
                    "8404 12th Ave S"
                </A>

                // Desktop nav
                <nav class="hidden md:flex items-center gap-6">
                    <A href="/" attr:class="hover:text-orange-400 transition-colors">"Property"</A>
                    <A href="/login" attr:class="bg-orange-600 hover:bg-orange-700 px-4 py-2 rounded-lg text-sm font-medium transition-colors">
                        "Sign In"
                    </A>
                </nav>

                // Mobile toggle
                <button
                    class="md:hidden text-white"
                    on:click=move |_| set_mobile_open.update(|v| *v = !*v)
                >
                    <svg class="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <Show
                            when=move || mobile_open.get()
                            fallback=|| view! {
                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 6h16M4 12h16M4 18h16" />
                            }
                        >
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
                        </Show>
                    </svg>
                </button>
            </div>

            // Mobile nav
            <Show when=move || mobile_open.get()>
                <nav class="md:hidden px-4 pb-4 space-y-2">
                    <A href="/" attr:class="block py-2 hover:text-orange-400">"Property"</A>
                    <A href="/login" attr:class="block py-2 text-orange-400">"Sign In"</A>
                </nav>
            </Show>
        </header>
    }
}
