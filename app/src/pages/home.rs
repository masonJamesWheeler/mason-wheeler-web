use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        // Full-bleed hero — no max-width constraint
        <div class="relative w-full h-[55vh] min-h-[400px]">
            <img
                src="/photos/LCS00733.jpg"
                alt="Living room at 8404 12th Ave S"
                class="absolute inset-0 w-full h-full object-cover"
            />
            <div class="absolute inset-0 bg-gradient-to-t from-stone-950/70 via-stone-900/20 to-stone-900/5" />
            <div class="absolute bottom-0 left-0 right-0 p-8 sm:p-12">
                <div class="max-w-5xl mx-auto">
                    <h1 class="text-3xl sm:text-4xl font-semibold text-white tracking-tight">"8404 12th Ave S"</h1>
                    <p class="text-white/60 text-sm sm:text-base mt-2">"Seattle, WA 98108"</p>
                </div>
            </div>
        </div>

        // CTA section
        <div class="max-w-5xl mx-auto px-6 py-16 text-center">
            <p class="text-stone-500 text-base mb-8">"Resident portal"</p>
            <a href="/login" class="btn-primary text-base px-10 py-3.5">
                "Sign in"
                <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                </svg>
            </a>
        </div>

        // Screening disclosure (required by WA state law)
        <div class="max-w-3xl mx-auto px-6 pb-12">
            <div class="border-t border-stone-200 pt-8">
                <h2 class="text-xs font-semibold uppercase tracking-wider text-stone-400 mb-3">"Tenant Screening Disclosure"</h2>
                <p class="text-sm text-stone-500 leading-relaxed">
                    "Per Washington State law (RCW 59.18.257), this property does not accept reusable tenant screening reports. "
                    "All applicants are screened through TransUnion SmartMove. Screening costs may be passed to the applicant. "
                    "Screening criteria are applied consistently to all applicants in accordance with Seattle\u{2019}s First-in-Time rule."
                </p>
            </div>
        </div>
    }
}
