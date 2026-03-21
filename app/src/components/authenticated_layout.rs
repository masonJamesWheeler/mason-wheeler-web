use leptos::prelude::*;

use crate::state::auth::provide_auth_state;
use crate::components::header::Header;
use crate::components::footer::Footer;

/// Wraps authenticated pages. Provides auth context and renders the
/// nav + footer chrome. Use for /tenant/* and /admin/* routes.
#[component]
pub fn AuthenticatedLayout(children: Children) -> impl IntoView {
    // Provide auth state to the entire subtree — Header reads it via use_context
    let _auth = provide_auth_state();

    // Evaluate children once at top level, outside any closures
    let content = children();

    view! {
        <Header />
        <main class="flex-1">
            {content}
        </main>
        <Footer />
    }
}
