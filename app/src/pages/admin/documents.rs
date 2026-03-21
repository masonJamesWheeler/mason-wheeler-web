#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Document;

#[cfg(feature = "hydrate")]
fn fetch_documents(set_documents: WriteSignal<Vec<Document>>) {
    leptos::task::spawn_local(async move {
        if let Ok(data) = crate::api::client::api_get::<Vec<Document>>("/api/documents").await {
            set_documents.set(data);
        }
    });
}

#[cfg(feature = "hydrate")]
fn upload_file(dtype: String, file: web_sys::File, set_documents: WriteSignal<Vec<Document>>) {
    leptos::task::spawn_local(async move {
        let form_data = web_sys::FormData::new().unwrap();
        let _ = form_data.append_with_blob_and_filename("file", &file, &file.name());
        let _ = form_data.append_with_str("doc_type", &dtype);
        let _ = form_data.append_with_str("name", &file.name());

        let opts = web_sys::RequestInit::new();
        opts.set_method("POST");
        opts.set_body(&form_data);

        let request = web_sys::Request::new_with_str_and_init("/api/documents/upload", &opts).unwrap();

        let window = web_sys::window().unwrap();
        if let Ok(resp_val) = wasm_bindgen_futures::JsFuture::from(window.fetch_with_request(&request)).await {
            let resp: web_sys::Response = resp_val.into();
            if resp.ok() {
                // Refresh documents list
                fetch_documents(set_documents);
            }
        }
    });
}

#[cfg(feature = "hydrate")]
fn delete_doc(id: String, set_documents: WriteSignal<Vec<Document>>) {
    leptos::task::spawn_local(async move {
        if crate::api::client::api_delete(&format!("/api/documents/{}", id)).await.is_ok() {
            fetch_documents(set_documents);
        }
    });
}

#[allow(unused)]
#[component]
pub fn AdminDocuments() -> impl IntoView {
    let (documents, set_documents) = signal::<Vec<Document>>(vec![]);

    #[cfg(feature = "hydrate")]
    {
        fetch_documents(set_documents);
    }

    let doc_types = vec![
        ("lease", "Lease Agreement"),
        ("lead_paint", "Lead-Based Paint Disclosure"),
        ("mold", "Mold Information"),
        ("checklist", "Move-In Condition Checklist"),
        ("deposit_receipt", "Security Deposit Receipt"),
        ("fee_disclosure", "Fee-in-Lieu-of-Deposit Disclosure"),
        ("contact_info", "Landlord Contact Information"),
    ];

    view! {
        <div class="max-w-5xl mx-auto px-6 py-10">
            <h1 class="text-3xl font-bold tracking-tight text-stone-900 mb-1">"Manage Documents"</h1>
            <p class="text-stone-500 mb-8">"Upload and manage required property documents"</p>

            // Legal compliance info box
            <div class="card !bg-amber-50 !border-amber-200 mb-8">
                <div class="flex gap-3">
                    <svg class="w-5 h-5 text-amber-500 mt-0.5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                        <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                    </svg>
                    <div>
                        <p class="font-semibold text-stone-800 text-sm">"WA State Required Disclosures (RCW 59.18)"</p>
                        <p class="text-stone-600 text-sm mt-1">"Upload all required documents before the tenant moves in. Lead paint disclosure is mandatory for this 1940s property."</p>
                    </div>
                </div>
            </div>

            // Document rows
            <div class="card !p-0 overflow-hidden">
                {doc_types.into_iter().enumerate().map(|(i, (dtype, label))| {
                    let dtype = dtype.to_string();
                    let label = label.to_string();
                    let dtype_for_status = dtype.clone();
                    let dtype_for_buttons = dtype.clone();
                    let is_last = i == 6;
                    let border_class = if is_last { "" } else { "border-b border-stone-200/60" };
                    let input_id = format!("file-input-{}", dtype);
                    let input_id_for_btn = input_id.clone();
                    let input_id_for_handler = input_id.clone();
                    view! {
                        <div class={format!("flex items-center justify-between px-5 py-4 hover:bg-stone-50/50 transition-colors {}", border_class)}>
                            <div class="min-w-0 flex-1">
                                <p class="font-medium text-stone-800">{label}</p>
                                {move || {
                                    let docs = documents.get();
                                    let found = docs.iter().find(|d| d.doc_type == dtype_for_status);
                                    match found {
                                        Some(doc) => {
                                            let name = doc.name.clone();
                                            view! {
                                                <div class="flex items-center gap-2 mt-1">
                                                    <span class="badge-success">"Uploaded"</span>
                                                    <span class="text-sm text-stone-500 truncate">{name}</span>
                                                </div>
                                            }.into_any()
                                        }
                                        None => view! {
                                            <div class="mt-1">
                                                <span class="badge-warning">"Not uploaded"</span>
                                            </div>
                                        }.into_any()
                                    }
                                }}
                            </div>
                            <div class="flex items-center gap-2 ml-4 shrink-0">
                                {move || {
                                    let docs = documents.get();
                                    let found = docs.iter().find(|d| d.doc_type == dtype_for_buttons).cloned();
                                    match found {
                                        Some(doc) => {
                                            let view_path = format!("/api/documents/file/{}", doc.id);
                                            let delete_id = doc.id.clone();
                                            view! {
                                                <a
                                                    href={view_path}
                                                    target="_blank"
                                                    class="btn-secondary text-sm"
                                                >
                                                    "View"
                                                </a>
                                                <button
                                                    class="btn-secondary text-sm !text-red-600 !border-red-200 hover:!bg-red-50"
                                                    on:click=move |_| {
                                                        #[cfg(feature = "hydrate")]
                                                        {
                                                            let id = delete_id.clone();
                                                            delete_doc(id, set_documents);
                                                        }
                                                    }
                                                >
                                                    "Delete"
                                                </button>
                                            }.into_any()
                                        }
                                        None => view! {
                                            <span />
                                        }.into_any()
                                    }
                                }}
                                // Hidden file input
                                <input
                                    type="file"
                                    id={input_id.clone()}
                                    style="display:none"
                                    on:change=move |ev| {
                                        #[cfg(feature = "hydrate")]
                                        {
                                            use wasm_bindgen::JsCast;
                                            let target = ev.target().unwrap();
                                            let input: web_sys::HtmlInputElement = target.unchecked_into();
                                            if let Some(files) = input.files() {
                                                if let Some(file) = files.get(0) {
                                                    let dtype = input.id().replace("file-input-", "");
                                                    upload_file(dtype, file, set_documents);
                                                }
                                            }
                                            // Reset the input so the same file can be re-selected
                                            input.set_value("");
                                        }
                                    }
                                />
                                <button
                                    class="btn-secondary text-sm"
                                    on:click=move |_| {
                                        #[cfg(feature = "hydrate")]
                                        {
                                            use wasm_bindgen::JsCast;
                                            let window = web_sys::window().unwrap();
                                            let document = window.document().unwrap();
                                            if let Some(el) = document.get_element_by_id(&input_id_for_btn) {
                                                let input: web_sys::HtmlInputElement = el.unchecked_into();
                                                input.click();
                                            }
                                        }
                                    }
                                >
                                    "Upload"
                                </button>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
