#![allow(unused)]

use leptos::prelude::*;

/// Typed-name signature component.
/// User types their full legal name, it renders in a script font as the "signature",
/// they check an agreement box, then click Sign.
/// Returns the typed name string to the callback.
#[component]
pub fn SignaturePad(
    /// Called with the typed legal name when the user signs
    on_sign: Callback<String>,
    /// Label shown above the signature line (e.g. "Tenant Signature")
    #[prop(default = "Signature".to_string(), into)]
    label: String,
    /// The document name being signed (e.g. "Residential Lease Agreement")
    #[prop(default = "this document".to_string(), into)]
    document_name: String,
) -> impl IntoView {
    let (typed_name, set_typed_name) = signal(String::new());
    let (agreed, set_agreed) = signal(false);

    let can_sign = move || !typed_name.get().trim().is_empty() && agreed.get();

    let handle_sign = move |_| {
        let name = typed_name.get().trim().to_string();
        if !name.is_empty() && agreed.get() {
            on_sign.run(name);
        }
    };

    view! {
        <div class="space-y-5">
            // Label
            <p class="text-sm font-medium text-stone-700">{label}</p>

            // Name input
            <div>
                <label class="block text-xs text-stone-500 mb-1.5 ml-1">"Type your full legal name"</label>
                <input
                    type="text"
                    class="input"
                    placeholder="e.g. John Michael Smith"
                    autocomplete="off"
                    on:input=move |ev| set_typed_name.set(event_target_value(&ev))
                    prop:value=move || typed_name.get()
                />
            </div>

            // Signature preview
            <Show when=move || !typed_name.get().trim().is_empty()>
                <div class="border border-stone-200 rounded-xl bg-white px-6 py-8 text-center">
                    <p class="text-xs text-stone-400 mb-3">"Signature preview"</p>
                    <p
                        class="text-3xl text-stone-900"
                        style="font-family: 'Dancing Script', 'Segoe Script', 'Brush Script MT', cursive; font-style: italic;"
                    >
                        {move || typed_name.get()}
                    </p>
                    <div class="mt-4 mx-auto w-64 border-b border-stone-300" />
                </div>
            </Show>

            // Agreement checkbox
            <label class="flex items-start gap-3 cursor-pointer select-none">
                <input
                    type="checkbox"
                    class="mt-0.5 h-4 w-4 rounded border-stone-300 text-stone-900 focus:ring-stone-500"
                    on:change=move |ev| {
                        set_agreed.set(event_target_checked(&ev));
                    }
                    prop:checked=move || agreed.get()
                />
                <span class="text-sm text-stone-600 leading-relaxed">
                    "By typing my name above and checking this box, I acknowledge that this constitutes \
                     my electronic signature on "
                    <span class="font-medium text-stone-900">{document_name}</span>
                    ", and I agree to be bound by its terms. I understand this has the same legal \
                     effect as a handwritten signature under the ESIGN Act (15 U.S.C. § 7001)."
                </span>
            </label>

            // Sign button
            <button
                type="button"
                class="btn-primary w-full py-3"
                disabled=move || !can_sign()
                on:click=handle_sign
            >
                "Sign Document"
            </button>
        </div>
    }
}
