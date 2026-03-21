use leptos::prelude::*;
use mason_wheeler_shared::Document;

#[component]
pub fn TenantDocuments() -> impl IntoView {
    let (documents, set_documents) = signal::<Vec<Document>>(vec![]);
    let (loading, set_loading) = signal(true);

    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            if let Ok(resp) = gloo_net::http::Request::get("/api/documents").send().await {
                if let Ok(data) = resp.json::<Vec<Document>>().await {
                    set_documents.set(data);
                }
            }
            set_loading.set(false);
        });
    }

    let required_disclosures = vec![
        ("Lead-Based Paint Disclosure", "lead_paint", "Required for homes built before 1978. Includes EPA pamphlet."),
        ("Mold Information", "mold", "Health hazards and prevention information per WA state law."),
        ("Move-In Condition Checklist", "checklist", "Documents the condition of the property at move-in."),
        ("Security Deposit Receipt", "deposit_receipt", "Receipt with depository name and location."),
        ("Fee-in-Lieu-of-Deposit Disclosure", "fee_disclosure", "Option to pay a fee instead of security deposit."),
        ("Landlord Contact Information", "contact_info", "Name, address, and contact details."),
    ];

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <h1 class="text-2xl font-bold text-slate-900 mb-6">"Documents & Disclosures"</h1>

            // Lease
            <div class="mb-8">
                <h2 class="text-lg font-semibold text-slate-900 mb-3">"Lease Agreement"</h2>
                <div class="bg-white border border-slate-200 rounded-xl p-6">
                    {move || {
                        let docs = documents.get();
                        let lease = docs.iter().find(|d| d.doc_type == "lease");
                        match lease {
                            Some(doc) => {
                                let path = doc.file_path.clone();
                                let name = doc.name.clone();
                                view! {
                                    <div class="flex items-center justify-between">
                                        <div>
                                            <p class="font-medium text-slate-900">{name}</p>
                                            <p class="text-sm text-slate-500">"Your current lease agreement"</p>
                                        </div>
                                        <a
                                            href={path}
                                            target="_blank"
                                            class="bg-slate-900 hover:bg-slate-800 text-white px-4 py-2 rounded-lg text-sm transition-colors"
                                        >
                                            "View / Download"
                                        </a>
                                    </div>
                                }.into_any()
                            }
                            None => view! {
                                <p class="text-slate-500">"No lease document uploaded yet."</p>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Required disclosures
            <h2 class="text-lg font-semibold text-slate-900 mb-3">"Required Disclosures (WA State Law)"</h2>
            <div class="space-y-3">
                {required_disclosures.into_iter().map(|(name, doc_type, desc)| {
                    let doc_type = doc_type.to_string();
                    let name = name.to_string();
                    let desc = desc.to_string();
                    view! {
                        <div class="bg-white border border-slate-200 rounded-xl p-4">
                            <div class="flex items-center justify-between">
                                <div>
                                    <p class="font-medium text-slate-900">{name}</p>
                                    <p class="text-sm text-slate-500">{desc}</p>
                                </div>
                                {move || {
                                    let docs = documents.get();
                                    let found = docs.iter().find(|d| d.doc_type == doc_type);
                                    match found {
                                        Some(doc) => {
                                            let path = doc.file_path.clone();
                                            view! {
                                                <a
                                                    href={path}
                                                    target="_blank"
                                                    class="bg-slate-100 hover:bg-slate-200 text-slate-700 px-3 py-1.5 rounded-lg text-sm transition-colors flex-shrink-0"
                                                >
                                                    "View"
                                                </a>
                                            }.into_any()
                                        }
                                        None => view! {
                                            <span class="text-xs text-yellow-600 bg-yellow-50 px-2 py-1 rounded-full flex-shrink-0">
                                                "Pending"
                                            </span>
                                        }.into_any()
                                    }
                                }}
                            </div>
                        </div>
                    }
                }).collect_view()}
            </div>

            <div class="mt-6 bg-blue-50 border border-blue-200 rounded-lg p-4 text-sm text-blue-800">
                <p class="font-medium mb-1">"Washington State Law (RCW 59.18)"</p>
                <p>"Your landlord is required to provide these disclosures. If any are marked as pending, please contact your landlord."</p>
            </div>
        </div>
    }
}
