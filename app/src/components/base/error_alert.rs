use leptos::prelude::*;

#[component]
pub fn ErrorAlert(
    #[prop(into)] message: Signal<Option<String>>,
    #[prop(optional)] on_close: Option<Callback<()>>,
) -> impl IntoView {
    view! {
        <Show when=move || message.get().is_some()>
            <div class="rounded-lg border border-red-200 bg-red-50 p-4 flex items-start gap-3">
                <svg class="h-5 w-5 text-red-400 flex-shrink-0 mt-0.5" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" d="M12 9v3.75m9-.75a9 9 0 11-18 0 9 9 0 0118 0zm-9 3.75h.008v.008H12v-.008z" />
                </svg>
                <p class="text-sm text-red-700 flex-1">{move || message.get().unwrap_or_default()}</p>
                {on_close.map(|cb| view! {
                    <button
                        class="text-red-400 hover:text-red-600 transition-colors"
                        on:click=move |_| cb.run(())
                    >
                        <svg class="h-4 w-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                        </svg>
                    </button>
                })}
            </div>
        </Show>
    }
}
