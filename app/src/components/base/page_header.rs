use leptos::prelude::*;

#[component]
pub fn PageHeader(
    #[prop(into)] title: String,
    #[prop(optional, into)] description: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    view! {
        <div class="flex flex-col gap-4 sm:flex-row sm:items-end sm:justify-between mb-8">
            <div>
                <h1 class="text-2xl font-semibold tracking-tight text-stone-900">{title}</h1>
                {description.map(|d| view! {
                    <p class="mt-1 text-stone-500">{d}</p>
                })}
            </div>
            {children.map(|c| view! {
                <div class="flex items-center gap-3 flex-shrink-0">
                    {c()}
                </div>
            })}
        </div>
    }
}
