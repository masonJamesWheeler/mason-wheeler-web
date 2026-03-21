use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="bg-slate-100 border-t border-slate-200 py-6 mt-auto">
            <div class="max-w-6xl mx-auto px-4 text-center text-sm text-slate-500">
                <p>"© 2026 Mason Wheeler — 8404 12th Ave S, Seattle, WA 98108"</p>
                <p class="mt-1">
                    "Questions? Contact "
                    <a href="mailto:mason@mason-wheeler.com" class="text-orange-600 hover:underline">
                        "mason@mason-wheeler.com"
                    </a>
                </p>
            </div>
        </footer>
    }
}
