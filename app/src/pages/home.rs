use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <div class="min-h-[85vh] flex items-center justify-center px-6">
            <div class="text-center max-w-sm">
                // Logo mark
                <div class="w-16 h-16 rounded-2xl bg-stone-900 flex items-center justify-center mx-auto mb-8 shadow-lg shadow-stone-900/10">
                    <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M2.25 12l8.954-8.955c.44-.439 1.152-.439 1.591 0L21.75 12M4.5 9.75v10.125c0 .621.504 1.125 1.125 1.125H9.75v-4.875c0-.621.504-1.125 1.125-1.125h2.25c.621 0 1.125.504 1.125 1.125V21h4.125c.621 0 1.125-.504 1.125-1.125V9.75M8.25 21h8.25" />
                    </svg>
                </div>

                <h1 class="text-2xl font-semibold text-stone-900 tracking-tight">"8404 12th Ave S"</h1>
                <p class="text-stone-400 mt-2 mb-10 text-[15px]">"Resident portal"</p>

                <a href="/login" class="btn-primary text-[15px] px-8 py-3">
                    "Sign in"
                    <svg class="w-4 h-4 ml-1" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                    </svg>
                </a>
            </div>
        </div>
    }
}
