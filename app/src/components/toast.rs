use leptos::prelude::*;

#[component]
pub fn Toast(
    message: ReadSignal<String>,
    toast_type: ReadSignal<String>,
    visible: ReadSignal<bool>,
    set_visible: WriteSignal<bool>,
) -> impl IntoView {
    // Auto-dismiss after 3 seconds when visible becomes true
    #[cfg(feature = "hydrate")]
    {
        let set_visible = set_visible.clone();
        Effect::new(move |_| {
            if visible.get() {
                let set_vis = set_visible.clone();
                leptos::task::spawn_local(async move {
                    gloo_timers::future::TimeoutFuture::new(3000).await;
                    set_vis.set(false);
                });
            }
        });
    }

    view! {
        <Show when=move || visible.get()>
            <div
                class=move || {
                    let base = "fixed top-4 right-4 z-50 px-5 py-3 rounded-xl shadow-lg border flex items-center gap-3 text-sm font-medium transition-all duration-300 animate-slide-in";
                    if toast_type.get() == "success" {
                        format!("{} bg-green-50 border-green-200 text-green-800", base)
                    } else {
                        format!("{} bg-red-50 border-red-200 text-red-800", base)
                    }
                }
            >
                <span class=move || {
                    if toast_type.get() == "success" {
                        "inline-flex items-center justify-center w-5 h-5 rounded-full bg-green-200 text-green-700"
                    } else {
                        "inline-flex items-center justify-center w-5 h-5 rounded-full bg-red-200 text-red-700"
                    }
                }>
                    {move || {
                        if toast_type.get() == "success" {
                            view! {
                                <svg class="w-3 h-3" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                </svg>
                            }.into_any()
                        } else {
                            view! {
                                <svg class="w-3 h-3" fill="none" stroke="currentColor" stroke-width="2.5" viewBox="0 0 24 24">
                                    <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            }.into_any()
                        }
                    }}
                </span>
                <span>{move || message.get()}</span>
                <button
                    class="ml-2 opacity-50 hover:opacity-100 transition-opacity"
                    on:click=move |_| set_visible.set(false)
                >
                    <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M6 18L18 6M6 6l12 12" />
                    </svg>
                </button>
            </div>
        </Show>
    }
}
