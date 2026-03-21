use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="min-h-[85vh] flex flex-col items-center justify-center px-6">
            // Hero image with overlay
            <div class="w-full max-w-2xl mb-10 rounded-2xl overflow-hidden shadow-xl shadow-stone-900/10 relative group">
                <img
                    src="/photos/LCS00733.jpg"
                    alt="Living room at 8404 12th Ave S"
                    class="w-full h-64 sm:h-80 object-cover"
                />
                <div class="absolute inset-0 bg-gradient-to-t from-stone-900/60 via-stone-900/10 to-transparent" />
                <div class="absolute bottom-0 left-0 right-0 p-6">
                    <h1 class="text-2xl font-semibold text-white tracking-tight">"8404 12th Ave S"</h1>
                    <p class="text-white/70 text-sm mt-1">"Seattle, WA 98108"</p>
                </div>
            </div>

            // CTA area
            <p class="text-stone-400 text-[15px] mb-6">"Resident portal"</p>
            <a href="/login" class="btn-primary text-[15px] px-8 py-3">
                "Sign in"
                <svg class="w-4 h-4 ml-1" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                </svg>
            </a>
        </div>
    }
}
