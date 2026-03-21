use leptos::prelude::*;

#[component]
pub fn ConfirmDialog(
    show: ReadSignal<bool>,
    #[prop(into)] title: Signal<String>,
    #[prop(into)] message: Signal<String>,
    #[prop(optional, into)] confirm_text: Option<String>,
    on_confirm: Callback<()>,
    on_cancel: Callback<()>,
) -> impl IntoView {
    let confirm_label = confirm_text.unwrap_or_else(|| "Confirm".to_string());

    view! {
        <Show when=move || show.get()>
            // Overlay
            <div
                class="fixed inset-0 z-50 bg-black/50 transition-opacity"
                on:click=move |_| on_cancel.run(())
            />
            // Dialog
            <div class="fixed inset-0 z-50 flex items-center justify-center p-4">
                <div
                    class="w-full max-w-md rounded-xl bg-white p-6 shadow-2xl"
                    on:click=move |e| e.stop_propagation()
                >
                    <h2 class="mb-2 text-lg font-semibold text-stone-900">{move || title.get()}</h2>
                    <p class="mb-6 text-sm text-stone-600">{move || message.get()}</p>
                    <div class="flex justify-end gap-3">
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-lg bg-white text-stone-700 border border-stone-300 hover:bg-stone-50 transition-colors"
                            on:click=move |_| on_cancel.run(())
                        >"Cancel"</button>
                        <button
                            class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-lg bg-red-600 text-white hover:bg-red-700 transition-colors"
                            on:click=move |_| on_confirm.run(())
                        >{confirm_label.clone()}</button>
                    </div>
                </div>
            </div>
        </Show>
    }
}
