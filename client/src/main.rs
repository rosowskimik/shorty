#[cfg(feature = "ssr")]
mod ssr {
    use std::time::Duration;

    use axum::{
        body::Body,
        extract::{Request, State},
        response::{IntoResponse, Response},
        routing::get,
        Router,
    };
    use clap::Parser;
    use client::{
        app::*,
        args::Cli,
        fileserv::file_and_error_handler,
        grpc::shorty_client::ShortyClient,
        intercept::TokenInterceptor,
        short::ssr::short_url_handler,
        state::{self, AppState},
    };
    use eyre::Context;
    use leptos::*;
    use leptos_axum::{
        generate_route_list, handle_server_fns_with_context, render_app_to_stream_with_context,
        LeptosRoutes,
    };
    use tokio::{net::TcpListener, signal};
    use tonic::transport::Endpoint;
    use tower::ServiceBuilder;
    use tower_http::{compression::CompressionLayer, timeout::TimeoutLayer};
    use tracing::{debug, info, Level};
    use tracing_subscriber::{filter::Targets, prelude::*};

    async fn server_fn_handler(
        State(app_state): State<AppState>,
        request: Request<Body>,
    ) -> impl IntoResponse {
        handle_server_fns_with_context(
            move || {
                provide_context(app_state.client.clone());
                provide_context(app_state.short_base.clone());
            },
            request,
        )
        .await
    }

    async fn leptos_routes_handler(
        State(app_state): State<AppState>,
        request: Request<Body>,
    ) -> Response {
        let handler = render_app_to_stream_with_context(
            app_state.leptos_options.clone(),
            move || {
                provide_context(app_state.client.clone());
            },
            App,
        );
        handler(request).await.into_response()
    }

    async fn init_grpc(uri: String, token: Option<String>) -> eyre::Result<state::ShortyClient> {
        const MAX_ATTEMPTS: usize = 5;

        let endpoint = Endpoint::from_shared(uri.clone()).wrap_err("Invalid gRPC URI")?;

        let mut attempt = 1;
        let channel = loop {
            match endpoint.connect().await {
                Err(e) => {
                    if attempt < MAX_ATTEMPTS {
                        debug!(
                            uri,
                            attempt, "Connecting to gRPC endpoint failed. Retrying..."
                        );
                        attempt += 1;
                        tokio::time::sleep(Duration::from_secs(15)).await;
                    } else {
                        return Err(e.into());
                    }
                }
                Ok(channel) => break channel,
            }
        };

        Ok(ShortyClient::with_interceptor(
            channel,
            TokenInterceptor { token },
        ))
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

        info!("Received shutdown request");
    }

    fn setup_tracing(lvl: &Level) {
        let filter = Targets::new()
            // For tracing from app use requested level
            .with_target("client", *lvl)
            // For all other, use >error
            .with_default(Level::ERROR);

        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(filter)
            .init();
    }

    pub async fn main() -> eyre::Result<()> {
        let args = Cli::parse();
        setup_tracing(&args.log);

        let conf = get_configuration(None).await?;
        let leptos_options = conf.leptos_options;
        let addr = leptos_options.site_addr;
        let routes = generate_route_list(App);

        debug!(endpoint = args.grpc, "Connecting to gRPC backend");
        let client = init_grpc(args.grpc, args.token)
            .await
            .wrap_err("Failed to connect to gRPC backend")?;

        let app_state = AppState {
            leptos_options,
            client,
            short_base: args.url.join("/s/").wrap_err("Parsing base URL failed")?,
        };

        debug!(leptos_options = ?app_state.leptos_options, short_base = app_state.short_base.to_string());

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

        let listener = TcpListener::bind(&addr).await.wrap_err("Failed to bind")?;

        info!(?addr, public_url = args.url.to_string(), "Starting server");
        axum::serve(listener, app.into_make_service())
            .with_graceful_shutdown(shutdown_signal())
            .await?;

        Ok(())
    }
}

#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() -> eyre::Result<()> {
    ssr::main().await
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
