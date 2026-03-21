#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Document;

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
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-slate-900 mb-6">"Manage Documents"</h1>

            <div class="bg-blue-50 border border-blue-200 rounded-lg p-4 text-sm text-blue-800 mb-6">
                <p class="font-medium">"WA State Required Disclosures (RCW 59.18)"</p>
                <p class="mt-1">"Upload all required documents before the tenant moves in. Lead paint disclosure is mandatory for this 1940s property."</p>
            </div>

            <div class="space-y-3">
                {doc_types.into_iter().map(|(dtype, label)| {
                    let dtype = dtype.to_string();
                    let label = label.to_string();
                    let dtype_clone = dtype.clone();
                    view! {
                        <div class="bg-white border border-slate-200 rounded-xl p-4">
                            <div class="flex items-center justify-between">
                                <div>
                                    <p class="font-medium text-slate-900">{label}</p>
                                    {move || {
                                        let docs = documents.get();
                                        let found = docs.iter().find(|d| d.doc_type == dtype);
                                        match found {
                                            Some(doc) => {
                                                let name = doc.name.clone();
                                                view! {
                                                    <p class="text-sm text-green-600">{format!("Uploaded: {}", name)}</p>
                                                }.into_any()
                                            }
                                            None => view! {
                                                <p class="text-sm text-yellow-600">"Not uploaded"</p>
                                            }.into_any()
                                        }
                                    }}
                                </div>
                                <div class="flex gap-2">
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
                                                        class="text-sm bg-slate-100 hover:bg-slate-200 text-slate-700 px-3 py-1.5 rounded-lg transition-colors"
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
                                    <button class="text-sm bg-orange-600 hover:bg-orange-700 text-white px-3 py-1.5 rounded-lg transition-colors">
                                        "Upload"
                                    </button>
                                </div>
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>
        </div>
    }
}
