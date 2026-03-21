use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonVariant {
    Primary,
    Secondary,
    Accent,
    Danger,
    Ghost,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ButtonSize {
    Sm,
    Md,
    Lg,
}

fn variant_classes(v: ButtonVariant) -> &'static str {
    match v {
        ButtonVariant::Primary => "bg-stone-900 text-white border border-stone-900 hover:bg-stone-800 focus:ring-stone-500 shadow-sm",
        ButtonVariant::Secondary => "bg-white text-stone-700 border border-stone-300 hover:border-stone-400 hover:bg-stone-50 focus:ring-stone-400",
        ButtonVariant::Accent => "bg-amber-500 text-white border border-amber-500 hover:bg-amber-600 focus:ring-amber-400 shadow-sm",
        ButtonVariant::Danger => "bg-white text-red-600 border border-red-200 hover:border-red-300 hover:bg-red-50 focus:ring-red-400",
        ButtonVariant::Ghost => "bg-transparent text-stone-600 hover:text-stone-800 hover:bg-stone-100",
    }
}

fn size_classes(s: ButtonSize) -> &'static str {
    match s {
        ButtonSize::Sm => "px-3 py-1.5 text-sm rounded-lg",
        ButtonSize::Md => "px-4 py-2 text-sm rounded-lg",
        ButtonSize::Lg => "px-6 py-3 text-base rounded-lg",
    }
}

#[component]
pub fn BaseButton(
    children: Children,
    #[prop(optional)] variant: Option<ButtonVariant>,
    #[prop(optional)] size: Option<ButtonSize>,
    #[prop(optional)] loading: Option<ReadSignal<bool>>,
    #[prop(optional)] disabled: Option<ReadSignal<bool>>,
    #[prop(optional)] on_click: Option<Callback<()>>,
    #[prop(optional, into)] href: Option<String>,
    #[prop(optional, into)] class: Option<String>,
    #[prop(optional, into)] button_type: Option<String>,
) -> impl IntoView {
    let v = variant.unwrap_or(ButtonVariant::Primary);
    let s = size.unwrap_or(ButtonSize::Md);
    let btype = button_type.unwrap_or_else(|| "button".to_string());

    let base = "inline-flex items-center justify-center font-medium transition-all duration-200 focus:outline-none focus:ring-2 focus:ring-offset-2 disabled:opacity-50 disabled:cursor-not-allowed active:scale-[0.98]";
    let combined = format!("{} {} {} {}", base, variant_classes(v), size_classes(s), class.unwrap_or_default());

    let is_disabled = move || {
        loading.map(|l| l.get()).unwrap_or(false) || disabled.map(|d| d.get()).unwrap_or(false)
    };

    let content = children();

    if let Some(url) = href {
        view! {
            <a href={url} class={combined}>
                {content}
            </a>
        }
        .into_any()
    } else {
        view! {
            <button
                type={btype}
                class={combined}
                disabled=move || is_disabled()
                on:click=move |_| {
                    if let Some(cb) = &on_click {
                        cb.run(());
                    }
                }
            >
                <Show when=move || loading.map(|l| l.get()).unwrap_or(false)>
                    <svg class="animate-spin -ml-1 mr-2 h-4 w-4" fill="none" viewBox="0 0 24 24">
                        <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                        <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                    </svg>
                </Show>
                {content}
            </button>
        }
        .into_any()
    }
}
