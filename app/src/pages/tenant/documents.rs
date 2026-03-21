#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::Document;

#[allow(unused)]
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
        <div class="max-w-5xl mx-auto px-6 py-10 font-['Inter']">
            <h1 class="text-3xl font-bold text-stone-900 tracking-tight mb-1">"Documents & Disclosures"</h1>
            <p class="text-stone-500 mb-8">"Access your lease, disclosures, and compliance documents."</p>

            // Lease Agreement Section
            <div class="mb-10">
                <h2 class="section-title text-sm font-semibold uppercase tracking-wider text-stone-400 mb-4">"Lease Agreement"</h2>
                <div class="card card-hover bg-white border border-stone-200 rounded-2xl p-6 shadow-sm hover:shadow-md transition-shadow">
                    {move || {
                        let docs = documents.get();
                        let lease = docs.iter().find(|d| d.doc_type == "lease");
                        match lease {
                            Some(doc) => {
                                let path = doc.file_path.clone();
                                let name = doc.name.clone();
                                view! {
                                    <div class="flex items-center gap-4">
                                        <div class="flex-shrink-0 w-11 h-11 rounded-xl bg-amber-50 flex items-center justify-center">
                                            <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-amber-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                            </svg>
                                        </div>
                                        <div class="flex-1 min-w-0">
                                            <p class="font-semibold text-stone-900">{name}</p>
                                            <p class="text-sm text-stone-500">"Your current lease agreement"</p>
                                        </div>
                                        <div class="flex items-center gap-3 flex-shrink-0">
                                            <span class="badge-success inline-flex items-center gap-1 text-xs font-medium text-emerald-700 bg-emerald-50 border border-emerald-200 px-2.5 py-1 rounded-full">
                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                                </svg>
                                                "Available"
                                            </span>
                                            <a
                                                href={path}
                                                target="_blank"
                                                class="btn-primary inline-flex items-center gap-1.5 bg-amber-500 hover:bg-amber-600 text-white font-medium px-4 py-2 rounded-lg text-sm transition-colors shadow-sm"
                                            >
                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-4 h-4" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4" />
                                                </svg>
                                                "View / Download"
                                            </a>
                                        </div>
                                    </div>
                                }.into_any()
                            }
                            None => view! {
                                <div class="flex items-center gap-4">
                                    <div class="flex-shrink-0 w-11 h-11 rounded-xl bg-stone-100 flex items-center justify-center">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-stone-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12h6m-6 4h6m2 5H7a2 2 0 01-2-2V5a2 2 0 012-2h5.586a1 1 0 01.707.293l5.414 5.414a1 1 0 01.293.707V19a2 2 0 01-2 2z" />
                                        </svg>
                                    </div>
                                    <p class="text-stone-500">"No lease document uploaded yet."</p>
                                </div>
                            }.into_any()
                        }
                    }}
                </div>
            </div>

            // Required Disclosures Section
            <div class="mb-10">
                <h2 class="section-title text-sm font-semibold uppercase tracking-wider text-stone-400 mb-4">"Required Disclosures (WA State Law)"</h2>
                <div class="space-y-3">
                    {required_disclosures.into_iter().map(|(name, doc_type, desc)| {
                        let doc_type = doc_type.to_string();
                        let name = name.to_string();
                        let desc = desc.to_string();
                        view! {
                            <div class="card card-hover bg-white border border-stone-200 rounded-2xl p-5 shadow-sm hover:shadow-md transition-shadow">
                                <div class="flex items-center gap-4">
                                    <div class="flex-shrink-0 w-10 h-10 rounded-xl bg-stone-50 border border-stone-100 flex items-center justify-center">
                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-stone-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12l2 2 4-4m5.618-4.016A11.955 11.955 0 0112 2.944a11.955 11.955 0 01-8.618 3.04A12.02 12.02 0 003 9c0 5.591 3.824 10.29 9 11.622 5.176-1.332 9-6.03 9-11.622 0-1.042-.133-2.052-.382-3.016z" />
                                        </svg>
                                    </div>
                                    <div class="flex-1 min-w-0">
                                        <p class="font-semibold text-stone-900">{name}</p>
                                        <p class="text-sm text-stone-500 leading-relaxed">{desc}</p>
                                    </div>
                                    <div class="flex-shrink-0">
                                        {move || {
                                            let docs = documents.get();
                                            let found = docs.iter().find(|d| d.doc_type == doc_type);
                                            match found {
                                                Some(doc) => {
                                                    let path = doc.file_path.clone();
                                                    view! {
                                                        <div class="flex items-center gap-3">
                                                            <span class="badge-success inline-flex items-center gap-1 text-xs font-medium text-emerald-700 bg-emerald-50 border border-emerald-200 px-2.5 py-1 rounded-full">
                                                                <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2.5">
                                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M5 13l4 4L19 7" />
                                                                </svg>
                                                                "Available"
                                                            </span>
                                                            <a
                                                                href={path}
                                                                target="_blank"
                                                                class="btn-secondary inline-flex items-center gap-1.5 bg-stone-100 hover:bg-stone-200 text-stone-700 font-medium px-3.5 py-1.5 rounded-lg text-sm transition-colors"
                                                            >
                                                                "View"
                                                            </a>
                                                        </div>
                                                    }.into_any()
                                                }
                                                None => view! {
                                                    <span class="badge-warning inline-flex items-center gap-1 text-xs font-medium text-amber-700 bg-amber-50 border border-amber-200 px-2.5 py-1 rounded-full">
                                                        <svg xmlns="http://www.w3.org/2000/svg" class="w-3.5 h-3.5" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                                                            <path stroke-linecap="round" stroke-linejoin="round" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                        </svg>
                                                        "Pending"
                                                    </span>
                                                }.into_any()
                                            }
                                        }}
                                    </div>
                                </div>
                            </div>
                        }
                    }).collect_view()}
                </div>
            </div>

            // Legal Compliance Notice
            <div class="bg-sky-50 border border-sky-100 rounded-2xl p-5">
                <div class="flex gap-3">
                    <div class="flex-shrink-0 mt-0.5">
                        <svg xmlns="http://www.w3.org/2000/svg" class="w-5 h-5 text-sky-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </div>
                    <div>
                        <p class="font-semibold text-sky-900 text-sm mb-1">"Washington State Law (RCW 59.18)"</p>
                        <p class="text-sm text-sky-700 leading-relaxed">"Your landlord is required to provide these disclosures. If any are marked as pending, please contact your landlord."</p>
                    </div>
                </div>
            </div>
        </div>
    }
}
