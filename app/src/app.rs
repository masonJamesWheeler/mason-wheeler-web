use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{components::*, path};

use crate::components::header::Header;
use crate::components::footer::Footer;
use crate::pages;

pub fn shell(options: leptos::config::LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <meta name="description" content="8404 12th Ave S - Rental Property in Seattle, WA" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body class="min-h-screen bg-white">
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/mason-wheeler-web.css" />
        <Title text="8404 12th Ave S | Seattle Rental" />

        <Router>
            <Header />
            <main class="flex-grow">
                <Routes fallback=|| view! { <p class="p-8 text-center">"Page not found."</p> }>
                    <Route path=path!("/") view=pages::home::HomePage />
                    <Route path=path!("/login") view=pages::login::LoginPage />
                    <Route path=path!("/tenant") view=pages::tenant::dashboard::TenantDashboard />
                    <Route path=path!("/tenant/payments") view=pages::tenant::payments::TenantPayments />
                    <Route path=path!("/tenant/documents") view=pages::tenant::documents::TenantDocuments />
                    <Route path=path!("/tenant/maintenance") view=pages::tenant::maintenance::TenantMaintenance />
                    <Route path=path!("/admin") view=pages::admin::dashboard::AdminDashboard />
                    <Route path=path!("/admin/payments") view=pages::admin::payments::AdminPayments />
                    <Route path=path!("/admin/documents") view=pages::admin::documents::AdminDocuments />
                    <Route path=path!("/admin/maintenance") view=pages::admin::maintenance::AdminMaintenance />
                </Routes>
            </main>
            <Footer />
        </Router>
    }
}
