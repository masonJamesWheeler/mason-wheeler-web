use leptos::prelude::*;

#[component]
pub fn EmptyState(
    #[prop(into)] title: String,
    #[prop(into)] message: String,
    #[prop(optional, into)] icon: Option<String>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let icon_path = icon.unwrap_or_else(|| {
        "M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z".to_string()
    });

    view! {
        <div class="flex flex-col items-center justify-center py-16 text-center">
            <div class="mx-auto mb-4 flex h-14 w-14 items-center justify-center rounded-full bg-stone-100">
                <svg class="h-7 w-7 text-stone-400" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" d={icon_path} />
                </svg>
            </div>
            <h3 class="text-lg font-semibold text-stone-900 mb-1">{title}</h3>
            <p class="text-sm text-stone-500 max-w-sm mx-auto mb-6">{message}</p>
            {children.map(|c| c())}
        </div>
    }
}
