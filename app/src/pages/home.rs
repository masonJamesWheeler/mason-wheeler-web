use leptos::prelude::*;

/// Landing page — just redirects to login or shows a simple welcome
#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="min-h-[80vh] flex items-center justify-center px-4">
            <div class="text-center max-w-md">
                <h1 class="text-3xl font-bold text-slate-900 mb-2">"8404 12th Ave S"</h1>
                <p class="text-slate-500 mb-8">"Rental Property Management"</p>
                <a
                    href="/login"
                    class="inline-block bg-orange-600 hover:bg-orange-700 text-white font-medium px-8 py-3 rounded-lg transition-colors"
                >
                    "Sign In"
                </a>
            </div>
        </div>
    }
}
