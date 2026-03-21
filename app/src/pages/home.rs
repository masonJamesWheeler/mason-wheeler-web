use leptos::prelude::*;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        // ── Mobile/Tablet: stacked hero ──────────────────────────────
        // ── Desktop (lg+): split layout — left 1/3 text, right 2/3 images
        <div class="lg:flex lg:min-h-[calc(100vh-3.5rem)]">

            // Left panel — CTA + disclosure (desktop only)
            <div class="hidden lg:flex lg:w-1/3 lg:flex-col lg:justify-center lg:px-12 xl:px-16">
                <div class="max-w-sm">
                    <p class="text-stone-400 text-sm mb-6">"Resident portal"</p>

                    <h1 class="text-3xl xl:text-4xl font-semibold text-stone-900 tracking-tight leading-tight">
                        "Welcome home."
                    </h1>
                    <p class="text-stone-500 mt-3 text-base leading-relaxed">
                        "Pay rent, view documents, and manage your lease — all in one place."
                    </p>

                    <div class="mt-8">
                        <a href="/login" class="flex items-center justify-center gap-2 w-full bg-stone-900 text-white font-medium py-3 rounded-xl hover:bg-stone-800 transition-all active:scale-[0.98]">
                            "Sign in"
                            <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                            </svg>
                        </a>
                    </div>

                    // Screening disclosure
                    <div class="mt-16 pt-6 border-t border-stone-200">
                        <p class="text-[10px] font-semibold uppercase tracking-wider text-stone-400 mb-2">"Screening Disclosure"</p>
                        <p class="text-xs text-stone-400 leading-relaxed">
                            "Per WA law (RCW 59.18.257), this property does not accept reusable screening reports. "
                            "Applicants are screened via TransUnion SmartMove."
                        </p>
                    </div>
                </div>
            </div>

            // Right panel — photo grid (desktop)
            <div class="hidden lg:block lg:w-2/3 lg:relative">
                <div class="absolute inset-0 grid grid-cols-2 grid-rows-2 gap-1">
                    <div class="col-span-2 relative overflow-hidden">
                        <img src="/photos/living-room-fireplace-wide.jpg" alt="Living room with fireplace" class="absolute inset-0 w-full h-full object-cover" />
                    </div>
                    <div class="relative overflow-hidden">
                        <img src="/photos/kitchen-dining-nook-wide.jpg" alt="Updated kitchen" class="absolute inset-0 w-full h-full object-cover" />
                    </div>
                    <div class="relative overflow-hidden">
                        <img src="/photos/bedroom-1-primary-dark-furniture.jpg" alt="Primary bedroom" class="absolute inset-0 w-full h-full object-cover" />
                    </div>
                </div>
                // Gallery link overlay
                <a href="/gallery" class="absolute bottom-4 right-4 z-10 inline-flex items-center gap-1.5 bg-white/90 backdrop-blur-sm text-stone-800 text-sm font-medium px-4 py-2 rounded-xl hover:bg-white transition-colors shadow-lg">
                    "View all 23 photos"
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                    </svg>
                </a>
            </div>

            // ── Mobile/Tablet: stacked layout ────────────────────────
            <div class="lg:hidden">
                <div class="relative w-full h-[55vh] min-h-[400px]">
                    <img src="/photos/living-room-fireplace-wide.jpg" alt="Living room at 8404 12th Ave S" class="absolute inset-0 w-full h-full object-cover" />
                    <div class="absolute inset-0 bg-gradient-to-t from-stone-950/70 via-stone-900/20 to-stone-900/5" />
                    <div class="absolute bottom-0 left-0 right-0 p-8 sm:p-12">
                        <div class="max-w-5xl mx-auto">
                            <h1 class="text-3xl sm:text-4xl font-semibold text-white tracking-tight">"8404 12th Ave S"</h1>
                            <p class="text-white/60 text-sm sm:text-base mt-2">"Seattle, WA 98108"</p>
                        </div>
                    </div>
                </div>

                <div class="max-w-5xl mx-auto px-6 py-12 text-center">
                    <p class="text-stone-500 text-base mb-6">"Resident portal"</p>
                    <a href="/login" class="inline-flex items-center justify-center gap-2 bg-stone-900 text-white font-medium px-10 py-3.5 rounded-xl hover:bg-stone-800 transition-all active:scale-[0.98]">
                        "Sign in"
                        <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M13.5 4.5L21 12m0 0l-7.5 7.5M21 12H3" />
                        </svg>
                    </a>
                </div>

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
            </div>
        </div>
    }
}
