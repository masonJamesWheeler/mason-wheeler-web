use leptos::prelude::*;

use crate::state::auth::provide_auth_state;
use crate::components::header::Header;
use crate::components::footer::Footer;

/// Wraps authenticated pages. Provides auth context and renders the
/// nav + footer chrome. Use for /tenant/* and /admin/* routes.
#[component]
pub fn AuthenticatedLayout(children: Children) -> impl IntoView {
    let auth = provide_auth_state();

    // Evaluate children once at top level — never inside closures
    let content = children();

    view! {
        <Header />
        <main class="flex-1">
            <Show
                when=move || !auth.loading.get()
                fallback=|| view! {
                    <div class="flex items-center justify-center min-h-[60vh]">
                        <svg class="animate-spin h-8 w-8 text-stone-400" fill="none" viewBox="0 0 24 24">
                            <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                            <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                        </svg>
                    </div>
                }
            >
                {content.clone()}
            </Show>
        </main>
        <Footer />
    }
}
