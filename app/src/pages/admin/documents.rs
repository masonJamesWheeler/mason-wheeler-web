#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Document;

#[allow(unused)]
#[component]
pub fn AdminDocuments() -> impl IntoView {
    let (documents, set_documents) = signal::<Vec<Document>>(vec![]);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/documents").send().await {
                if let Ok(data) = resp.json::<Vec<Document>>().await {
                    set_documents.set(data);
                }
            }
        });
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
                    let dtype_clone = dtype.clone();
                    let is_last = i == 6;
                    let border_class = if is_last { "" } else { "border-b border-stone-200/60" };
                    view! {
                        <div class={format!("flex items-center justify-between px-5 py-4 hover:bg-stone-50/50 transition-colors {}", border_class)}>
                            <div class="min-w-0 flex-1">
                                <p class="font-medium text-stone-800">{label}</p>
                                {move || {
                                    let docs = documents.get();
                                    let found = docs.iter().find(|d| d.doc_type == dtype);
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
                                    let found = docs.iter().find(|d| d.doc_type == dtype_clone);
                                    match found {
                                        Some(doc) => {
                                            let path = doc.file_path.clone();
                                            view! {
                                                <a
                                                    href={path}
                                                    target="_blank"
                                                    class="btn-secondary text-sm"
                                                >
                                                    "View"
                                                </a>
                                            }.into_any()
                                        }
                                        None => view! {
                                            <span />
                                        }.into_any()
                                    }
                                }}
                                // TODO: file upload button
                                <button class="btn-secondary text-sm">
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
