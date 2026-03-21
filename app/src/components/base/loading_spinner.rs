use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum SpinnerSize {
    Sm,
    Md,
    Lg,
}

#[component]
pub fn LoadingSpinner(
    #[prop(optional)] size: Option<SpinnerSize>,
    #[prop(optional, into)] text: Option<String>,
) -> impl IntoView {
    let sz = size.unwrap_or(SpinnerSize::Md);
    let dim = match sz {
        SpinnerSize::Sm => "h-5 w-5",
        SpinnerSize::Md => "h-8 w-8",
        SpinnerSize::Lg => "h-12 w-12",
    };

    view! {
        <div class="flex flex-col items-center justify-center py-12">
            <svg class={format!("animate-spin text-stone-400 {}", dim)} fill="none" viewBox="0 0 24 24">
                <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
            </svg>
            {text.map(|t| view! {
                <p class="mt-3 text-sm text-stone-500">{t}</p>
            })}
        </div>
    }
}
