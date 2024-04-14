use leptos::*;
use url::Url;

#[cfg(feature = "ssr")]
pub mod ssr {
    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::Redirect,
    };
    use leptos::{use_context, ServerFnError};
    use tonic::Code;
    use tracing::{error, instrument};
    use url::Url;

    use crate::{grpc::SlugMessage, state::ShortyClient};

    #[instrument(level = "trace", skip_all, fields(slug = slug))]
    pub async fn short_url_handler(
        State(mut client): State<ShortyClient>,
        Path(slug): Path<String>,
    ) -> Result<Redirect, (StatusCode, &'static str)> {
        match client.get_original(SlugMessage { slug }).await {
            Ok(resp) => {
                let msg = resp.into_inner();
                Ok(Redirect::to(&msg.url))
            }
            Err(e) if e.code() == Code::NotFound => Err((StatusCode::NOT_FOUND, "Not found")),
            Err(_) => Err((StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")),
        }
    }

    pub fn client() -> Result<ShortyClient, ServerFnError> {
        use_context::<ShortyClient>().ok_or_else(|| {
            error!("gRPC client not in context");
            ServerFnError::ServerError("Something went wrong".to_string())
        })
    }

    pub fn short_base() -> Result<Url, ServerFnError> {
        use_context::<Url>().ok_or_else(|| {
            error!("Server URL not in context!");
            ServerFnError::ServerError("Something went wrong".to_string())
        })
    }
}

#[server(ShortenUrl)]
pub async fn shorten_url(url: Url) -> Result<Url, ServerFnError> {
    use leptos::server_fn::error::NoCustomError;

    use crate::grpc::UrlMessage;

    use ssr::{client, short_base};

    if url.scheme().is_empty() || url.cannot_be_a_base() {
        return Err(ServerFnError::ServerError("Bad URL".to_string()));
    }

    let mut client = client()?;
    let resp = client
        .shorten(UrlMessage {
            url: url.to_string(),
        })
        .await
        .map_err(|e| ServerFnError::<NoCustomError>::ServerError(e.message().to_string()))?;

    let msg = resp.into_inner();
    short_base()?
        .join(&msg.slug)
        .map_err(|_| ServerFnError::ServerError("Something went wrong".to_string()))
}
