use leptos::prelude::*;

#[component]
pub fn FormField(
    children: Children,
    #[prop(into)] label: String,
    #[prop(optional, into)] field_id: Option<String>,
    #[prop(optional)] required: Option<bool>,
    #[prop(optional, into)] hint: Option<String>,
    #[prop(optional)] error: Option<Signal<Option<String>>>,
) -> impl IntoView {
    view! {
        <div class="space-y-1.5">
            <label
                for={field_id.unwrap_or_default()}
                class="block text-sm font-medium text-stone-700"
            >
                {label}
                {required.unwrap_or(false).then(|| view! {
                    <span class="ml-1 text-red-500">"*"</span>
                })}
            </label>
            {hint.map(|h| view! {
                <p class="text-sm text-stone-500">{h}</p>
            })}
            {children()}
            {move || error.and_then(|e| e.get()).map(|msg| view! {
                <p class="text-sm text-red-600">{msg}</p>
            })}
        </div>
    }
}
