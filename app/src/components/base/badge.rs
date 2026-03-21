use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq)]
pub enum BadgeVariant {
    Success,
    Warning,
    Error,
    Info,
    Neutral,
}

fn badge_classes(v: BadgeVariant) -> &'static str {
    match v {
        BadgeVariant::Success => "bg-emerald-50 text-emerald-700 ring-1 ring-emerald-600/10",
        BadgeVariant::Warning => "bg-amber-50 text-amber-700 ring-1 ring-amber-600/10",
        BadgeVariant::Error => "bg-red-50 text-red-700 ring-1 ring-red-600/10",
        BadgeVariant::Info => "bg-sky-50 text-sky-700 ring-1 ring-sky-600/10",
        BadgeVariant::Neutral => "bg-stone-100 text-stone-600 ring-1 ring-stone-500/10",
    }
}

#[component]
pub fn Badge(
    children: Children,
    #[prop(optional)] variant: Option<BadgeVariant>,
) -> impl IntoView {
    let v = variant.unwrap_or(BadgeVariant::Neutral);

    view! {
        <span class={format!(
            "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium {}",
            badge_classes(v)
        )}>
            {children()}
        </span>
    }
}
