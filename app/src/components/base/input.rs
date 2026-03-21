use leptos::prelude::*;

#[component]
pub fn BaseInput(
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional, into)] input_type: Option<String>,
    #[prop(optional, into)] autocomplete: Option<String>,
    #[prop(optional)] error: Option<Signal<Option<String>>>,
    #[prop(optional)] required: Option<bool>,
) -> impl IntoView {
    let itype = input_type.unwrap_or_else(|| "text".to_string());
    let has_error = move || error.and_then(|e| e.get()).is_some();

    let classes = move || {
        let base = "w-full rounded-lg px-4 py-2.5 text-stone-900 placeholder-stone-400 outline-none transition-all duration-200 focus:ring-2 focus:ring-offset-0";
        if has_error() {
            format!("{} border border-red-500 focus:border-red-500 focus:ring-red-500/20", base)
        } else {
            format!("{} border border-stone-300 focus:border-stone-500 focus:ring-stone-500/20", base)
        }
    };

    view! {
        <input
            type={itype}
            id={id.unwrap_or_default()}
            placeholder={placeholder.unwrap_or_default()}
            autocomplete={autocomplete.unwrap_or_default()}
            required={required.unwrap_or(false)}
            class=move || classes()
            on:input=move |ev| value.set(event_target_value(&ev))
            prop:value=move || value.get()
        />
    }
}

#[component]
pub fn BaseTextarea(
    value: RwSignal<String>,
    #[prop(optional, into)] placeholder: Option<String>,
    #[prop(optional, into)] id: Option<String>,
    #[prop(optional)] rows: Option<u32>,
    #[prop(optional)] required: Option<bool>,
) -> impl IntoView {
    let r = rows.unwrap_or(4);

    view! {
        <textarea
            id={id.unwrap_or_default()}
            placeholder={placeholder.unwrap_or_default()}
            rows={r}
            required={required.unwrap_or(false)}
            class="w-full rounded-lg px-4 py-2.5 text-stone-900 placeholder-stone-400 border border-stone-300 outline-none transition-all duration-200 focus:border-stone-500 focus:ring-2 focus:ring-stone-500/20 resize-vertical"
            on:input=move |ev| value.set(event_target_value(&ev))
            prop:value=move || value.get()
        />
    }
}
