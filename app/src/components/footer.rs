use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-stone-200/60 mt-auto">
            <div class="max-w-5xl mx-auto px-6 py-6 flex flex-col sm:flex-row items-center justify-between gap-3">
                <p class="text-xs text-stone-400">
                    "© 2026 · 8404 12th Ave S, Seattle, WA 98108"
                </p>
                <a href="mailto:masonwheeler@fieldflow.us" class="text-xs text-stone-400 hover:text-stone-600 transition-colors">
                    "Contact"
                </a>
            </div>
        </footer>
    }
}
