use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::components::authenticated_layout::AuthenticatedLayout;
use crate::pages;

pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="description" content="8404 12th Ave S - Resident portal" />
                <link rel="icon" type="image/svg+xml" href="/favicon.svg" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="min-h-screen flex flex-col bg-stone-50 text-stone-900">
                <App />
            </body>
        </html>
    }
}

/// Public page wrapper — no auth context
#[component]
fn PublicLayout(children: Children) -> impl IntoView {
    let content = children();
    view! {
        <Header />
        <main class="flex-1">
            {content}
        </main>
        <Footer />
    }
}

/// Protected page wrapper — auth context provided, header shows nav
#[component]
fn ProtectedLayout(children: Children) -> impl IntoView {
    let content = children();
    view! {
        <AuthenticatedLayout>
            {content}
        </AuthenticatedLayout>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/mason-wheeler-web.css" />
        <Title text="8404 12th Ave S | Seattle Rental" />

        <Router>
            <Routes fallback=|| view! {
                <PublicLayout>
                    <div class="flex items-center justify-center min-h-[60vh]">
                        <div class="text-center">
                            <h1 class="text-2xl font-semibold text-stone-900 mb-2">"Page not found"</h1>
                            <p class="text-stone-500 mb-6">"The page you're looking for doesn't exist."</p>
                            <a href="/" class="inline-flex items-center justify-center px-4 py-2 text-sm font-medium rounded-lg bg-stone-900 text-white hover:bg-stone-800 transition-colors">"Go home"</a>
                        </div>
                    </div>
                </PublicLayout>
            }>
                // Public routes — no auth required
                <Route path=path!("/") view=|| view! { <PublicLayout><pages::home::HomePage /></PublicLayout> } />
                <Route path=path!("/login") view=|| view! { <PublicLayout><pages::login::LoginPage /></PublicLayout> } />
                <Route path=path!("/forgot-password") view=|| view! { <PublicLayout><pages::forgot_password::ForgotPasswordPage /></PublicLayout> } />
                <Route path=path!("/reset-password") view=|| view! { <PublicLayout><pages::reset_password::ResetPasswordPage /></PublicLayout> } />
                <Route path=path!("/gallery") view=|| view! { <PublicLayout><pages::gallery::GalleryPage /></PublicLayout> } />
                <Route path=path!("/apply") view=|| view! { <PublicLayout><pages::apply::ApplyPage /></PublicLayout> } />

                // Tenant routes — auth required
                <Route path=path!("/tenant") view=|| view! { <ProtectedLayout><pages::tenant::dashboard::TenantDashboard /></ProtectedLayout> } />
                <Route path=path!("/tenant/payments") view=|| view! { <ProtectedLayout><pages::tenant::payments::TenantPayments /></ProtectedLayout> } />
                <Route path=path!("/tenant/documents") view=|| view! { <ProtectedLayout><pages::tenant::documents::TenantDocuments /></ProtectedLayout> } />
                <Route path=path!("/tenant/maintenance") view=|| view! { <ProtectedLayout><pages::tenant::maintenance::TenantMaintenance /></ProtectedLayout> } />
                <Route path=path!("/tenant/sign-lease") view=|| view! { <ProtectedLayout><pages::tenant::sign_lease::SignLeasePage /></ProtectedLayout> } />

                // Admin routes — auth required
                <Route path=path!("/admin") view=|| view! { <ProtectedLayout><pages::admin::dashboard::AdminDashboard /></ProtectedLayout> } />
                <Route path=path!("/admin/payments") view=|| view! { <ProtectedLayout><pages::admin::payments::AdminPayments /></ProtectedLayout> } />
                <Route path=path!("/admin/documents") view=|| view! { <ProtectedLayout><pages::admin::documents::AdminDocuments /></ProtectedLayout> } />
                <Route path=path!("/admin/maintenance") view=|| view! { <ProtectedLayout><pages::admin::maintenance::AdminMaintenance /></ProtectedLayout> } />
                <Route path=path!("/admin/tenants") view=|| view! { <ProtectedLayout><pages::admin::tenants::AdminTenants /></ProtectedLayout> } />
                <Route path=path!("/admin/reports") view=|| view! { <ProtectedLayout><pages::admin::reports::AdminReports /></ProtectedLayout> } />
                <Route path=path!("/admin/applicants") view=|| view! { <ProtectedLayout><pages::admin::applicants::AdminApplicants /></ProtectedLayout> } />
            </Routes>
        </Router>
    }
}
