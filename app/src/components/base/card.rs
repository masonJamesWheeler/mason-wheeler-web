use leptos::prelude::*;

#[component]
pub fn BaseCard(
    children: Children,
    #[prop(optional)] hoverable: Option<bool>,
    #[prop(optional, into)] class: Option<String>,
) -> impl IntoView {
    let hover = if hoverable.unwrap_or(false) {
        "hover:shadow-md hover:border-stone-300 hover:-translate-y-0.5 cursor-pointer"
    } else {
        ""
    };

    let classes = format!(
        "rounded-xl border border-stone-200 bg-white p-6 shadow-sm transition-all duration-200 {} {}",
        hover,
        class.unwrap_or_default()
    );

    view! {
        <div class={classes}>
            {children()}
        </div>
    }
}
