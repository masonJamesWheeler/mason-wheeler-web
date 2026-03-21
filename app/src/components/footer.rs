use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="border-t border-stone-200/60 mt-auto">
            <div class="max-w-5xl mx-auto px-6 py-8 flex flex-col sm:flex-row items-center justify-between gap-4">
                <div class="flex items-center gap-3">
                    <div class="w-6 h-6 rounded-md bg-stone-900 flex items-center justify-center">
                        <svg class="w-3 h-3 text-white" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                        </svg>
                    </div>
                    <p class="text-xs text-stone-400">"8404 12th Ave S, Seattle, WA 98108"</p>
                </div>
                <a href="mailto:masonwheeler@fieldflow.us" class="text-xs text-stone-400 hover:text-stone-600 transition-colors">
                    "masonwheeler@fieldflow.us"
                </a>
            </div>
        </footer>
    }
}
