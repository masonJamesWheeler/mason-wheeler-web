#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use leptos::prelude::*;
    use leptos_axum::{file_and_error_handler, generate_route_list, LeptosRoutes};
    use mason_wheeler_app::app::{shell, App};
    use tower_http::services::ServeDir;
    use tracing_subscriber::{fmt, EnvFilter};

    dotenvy::dotenv().ok();

    // Init tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let _ = any_spawner::Executor::init_tokio();

    // Init database
    mason_wheeler_server::db::init_db();

    let conf = leptos::config::get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options.clone();
    let addr = leptos_options.site_addr;

    // Generate route list for Leptos SSR
    let routes = generate_route_list(App);

    // Build API router
    let api_router = mason_wheeler_server::routes::api_router();

    // Serve photos from the data directory
    let photos_service = ServeDir::new("data/photos");

    let leptos_router = Router::<leptos::config::LeptosOptions>::new()
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(file_and_error_handler(shell))
        .with_state(leptos_options);

    let app = leptos_router
        .nest("/api", api_router)
        .nest_service("/photos", photos_service);

    tracing::info!("Starting server at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
fn main() {}
