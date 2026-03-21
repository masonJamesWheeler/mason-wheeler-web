#![allow(unused)]
use leptos::prelude::*;
use mason_wheeler_shared::LeaseAgreement;
use crate::components::signature_pad::SignaturePad;

#[component]
pub fn SignLeasePage() -> impl IntoView {
    let (lease, set_lease) = signal::<Option<LeaseAgreement>>(None);
    let (loading, set_loading) = signal(true);
    let (signed, set_signed) = signal(false);
    let (error, set_error) = signal::<Option<String>>(None);

    // Fetch the tenant's lease
    #[cfg(feature = "hydrate")]
    {
        leptos::task::spawn_local(async move {
            match crate::api::client::api_get::<Option<LeaseAgreement>>("/api/tenant/lease").await {
                Ok(data) => set_lease.set(data),
                Err(e) => set_error.set(Some(e.message)),
            }
            set_loading.set(false);
        });
    }

    let on_sign = Callback::new(move |name: String| {
        #[cfg(feature = "hydrate")]
        {
            let lease_data = lease.get();
            if let Some(l) = lease_data {
                let lease_id = l.id.clone();
                leptos::task::spawn_local(async move {
                    let body = mason_wheeler_shared::SignLeaseRequest {
                        lease_id,
                        full_legal_name: name,
                    };
                    match crate::api::client::api_post::<serde_json::Value>(
                        "/api/tenant/lease/sign",
                        &body,
                    )
                    .await
                    {
                        Ok(_) => set_signed.set(true),
                        Err(e) => set_error.set(Some(e.message)),
                    }
                });
            }
        }
    });

    view! {
        <div class="max-w-3xl mx-auto px-6 py-10">
            // Loading
            <Show when=move || loading.get()>
                <div class="flex items-center justify-center py-20">
                    <div class="w-8 h-8 border-2 border-stone-200 border-t-amber-500 rounded-full animate-spin" />
                </div>
            </Show>

            // Error
            <Show when=move || error.get().is_some()>
                <div class="mb-6 px-4 py-3 rounded-xl bg-red-50 border border-red-100">
                    <p class="text-sm text-red-700">{move || error.get().unwrap_or_default()}</p>
                </div>
            </Show>

            // No lease assigned
            <Show when=move || !loading.get() && lease.get().is_none() && error.get().is_none()>
                <div class="text-center py-20">
                    <div class="w-16 h-16 rounded-2xl bg-stone-100 flex items-center justify-center mx-auto mb-5">
                        <svg class="w-8 h-8 text-stone-300" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m0 12.75h7.5m-7.5 3H12M10.5 2.25H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                        </svg>
                    </div>
                    <p class="text-stone-900 font-medium mb-1">"No lease available"</p>
                    <p class="text-sm text-stone-500">"Your landlord hasn't prepared a lease agreement yet."</p>
                </div>
            </Show>

            // Successfully signed
            <Show when=move || signed.get()>
                <div class="text-center py-20">
                    <div class="w-16 h-16 rounded-2xl bg-emerald-50 flex items-center justify-center mx-auto mb-5">
                        <svg class="w-8 h-8 text-emerald-600" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                        </svg>
                    </div>
                    <h2 class="text-xl font-semibold text-stone-900 mb-2">"Lease Signed"</h2>
                    <p class="text-stone-500 mb-6">"Your signature has been recorded. Your landlord will countersign to finalize."</p>
                    <a href="/tenant/documents" class="btn-primary">"View Documents"</a>
                </div>
            </Show>

            // Lease ready to sign
            <Show when=move || {
                !loading.get() && !signed.get() && lease.get().is_some()
            }>
                {move || {
                    let l = lease.get();
                    l.map(|lease| {
                        let already_signed = lease.tenant_signed_name.is_some();
                        let status = lease.status.clone();
                        let start = lease.lease_start.clone().unwrap_or_default();
                        let end = lease.lease_end.clone().unwrap_or_default();
                        let rent = lease.rent_amount;
                        let lease_id = lease.id.clone();

                        view! {
                            <div>
                                <a href="/tenant/documents" class="text-sm text-stone-400 hover:text-stone-600 transition-colors">"← Documents"</a>
                                <h1 class="text-2xl font-semibold text-stone-900 tracking-tight mt-3 mb-1">"Residential Lease Agreement"</h1>
                                <p class="text-stone-500 text-sm mb-8">"8404 12th Ave S, Seattle, WA 98108"</p>

                                // Lease summary card
                                <div class="card p-6 mb-8">
                                    <div class="grid grid-cols-2 gap-4 text-sm">
                                        <div>
                                            <p class="stat-label">"Monthly Rent"</p>
                                            <p class="font-semibold text-stone-900 mt-1">{format!("${:.2}", rent)}</p>
                                        </div>
                                        <div>
                                            <p class="stat-label">"Lease Term"</p>
                                            <p class="font-semibold text-stone-900 mt-1">"12 months"</p>
                                        </div>
                                        <div>
                                            <p class="stat-label">"Start Date"</p>
                                            <p class="font-semibold text-stone-900 mt-1">{start}</p>
                                        </div>
                                        <div>
                                            <p class="stat-label">"End Date"</p>
                                            <p class="font-semibold text-stone-900 mt-1">{end}</p>
                                        </div>
                                    </div>
                                    <div class="mt-4 pt-4 border-t border-stone-200">
                                        <a
                                            href={format!("/api/pdf/lease/{}", lease_id)}
                                            target="_blank"
                                            class="btn-secondary text-sm"
                                        >
                                            <svg class="w-4 h-4" fill="none" stroke="currentColor" stroke-width="1.5" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" d="M19.5 14.25v-2.625a3.375 3.375 0 00-3.375-3.375h-1.5A1.125 1.125 0 0113.5 7.125v-1.5a3.375 3.375 0 00-3.375-3.375H8.25m.75 12l3 3m0 0l3-3m-3 3v-6m-1.5-9H5.625c-.621 0-1.125.504-1.125 1.125v17.25c0 .621.504 1.125 1.125 1.125h12.75c.621 0 1.125-.504 1.125-1.125V11.25a9 9 0 00-9-9z" />
                                            </svg>
                                            "Download Full Lease (PDF)"
                                        </a>
                                    </div>
                                </div>

                                // Status or signature
                                {if already_signed {
                                    view! {
                                        <div class="card p-6 bg-emerald-50 border-emerald-200">
                                            <div class="flex items-center gap-3">
                                                <svg class="w-5 h-5 text-emerald-600" fill="none" stroke="currentColor" stroke-width="2" viewBox="0 0 24 24">
                                                    <path stroke-linecap="round" stroke-linejoin="round" d="M9 12.75L11.25 15 15 9.75M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
                                                </svg>
                                                <div>
                                                    <p class="font-medium text-emerald-900">"You have signed this lease"</p>
                                                    <p class="text-sm text-emerald-700">
                                                        {if status == mason_wheeler_shared::LeaseStatus::Executed {
                                                            "Lease is fully executed. Both parties have signed."
                                                        } else {
                                                            "Awaiting landlord countersignature."
                                                        }}
                                                    </p>
                                                </div>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else {
                                    view! {
                                        <div class="card p-6">
                                            <SignaturePad
                                                on_sign=on_sign
                                                label="Tenant Signature".to_string()
                                                document_name="Residential Lease Agreement for 8404 12th Ave S".to_string()
                                            />
                                        </div>
                                    }.into_any()
                                }}
                            </div>
                        }
                    })
                }}
            </Show>
        </div>
    }
}
