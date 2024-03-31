#[cfg(feature = "ssr")]
mod ssr {
    use std::{env, time::Duration};

    use axum::{
        body::Body as AxumBody,
        extract::State,
        http::Request,
        response::{IntoResponse, Response},
        routing::get,
        Router,
    };
    use leptos::*;
    use leptos_axum::{
        generate_route_list, handle_server_fns_with_context, render_app_to_stream_with_context,
        LeptosRoutes,
    };
    use tokio::signal;
    use tower::ServiceBuilder;
    use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer};

    use shorty::{
        app::*, db::init_db_pool, env::PUB_URL_ENV, fileserv::file_and_error_handler, short::ssr::short_url_handler, state::AppState
    };
    use url::Url;

    async fn server_fn_handler(
        State(app_state): State<AppState>,
        request: Request<AxumBody>,
    ) -> impl IntoResponse {
        handle_server_fns_with_context(
            move || {
                provide_context(app_state.pool.clone());
                provide_context(app_state.public_base.clone())
            },
            request,
        )
        .await
    }

    async fn leptos_routes_handler(
        State(app_state): State<AppState>,
        request: Request<AxumBody>,
    ) -> Response {
        let handler = render_app_to_stream_with_context(
            app_state.leptos_options.clone(),
            move || {
                provide_context(app_state.pool.clone());
            },
            App,
        );
        handler(request).await.into_response()
    }

    async fn shutdown_signal() {
        let ctrl_c = async {
            signal::ctrl_c()
                .await
                .expect("failed to install Ctrl+C handler");
        };

        #[cfg(unix)]
        let terminate = async {
            signal::unix::signal(signal::unix::SignalKind::terminate())
                .expect("failed to install signal handler")
                .recv()
                .await;
        };

        #[cfg(not(unix))]
        let terminate = std::future::pending::<()>();

        tokio::select! {
            _ = ctrl_c => {},
            _ = terminate => {},
        };

        tracing::info!("Received shutdown request");
    }

    pub async fn main() {
        tracing_subscriber::fmt::init();

        let conf = get_configuration(None).await.unwrap();
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(App);

        let pool = init_db_pool().await.expect("Setting up DB failed");

        let public_base = {
            #[cfg(debug_assertions)]
            let url = env::var(PUB_URL_ENV).unwrap_or("http://localhost:3000".to_string());
            #[cfg(not(debug_assertions))]
            let url = env::var(PUB_URL_ENV).expect(&format!("Public server URL '{}' not set in environment!", PUB_URL_ENV));

            let url = Url::parse(&url).expect("Invalid public server URL format");

            if url.scheme().is_empty() || url.cannot_be_a_base() {
                panic!("Not absolute public server URL");
            }
            url
        };
        let app_state = AppState {
            leptos_options,
            pool,
            public_base,
        };

        // build our application with a route
        let app = Router::new()
            .route("/s/:slug", get(short_url_handler))
            .route(
                "/api/*fn_name",
                get(server_fn_handler).post(server_fn_handler),
            )
            .leptos_routes_with_handler(routes, get(leptos_routes_handler))
            .fallback(file_and_error_handler)
            .layer(
                ServiceBuilder::new()
                    .layer(CompressionLayer::new())
                    .layer(TimeoutLayer::new(Duration::from_secs(10))),
            )
            .with_state(app_state);

        let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
        tracing::info!("listening on {}", listener.local_addr().unwrap());
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await
            .unwrap();
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    ssr::main().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
