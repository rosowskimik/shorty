use axum::extract::FromRef;
use leptos::LeptosOptions;
use tonic::{service::interceptor::InterceptedService, transport::Channel};
use url::Url;

use crate::{grpc::shorty_client, intercept::TokenInterceptor};

pub type ShortyClient = shorty_client::ShortyClient<InterceptedService<Channel, TokenInterceptor>>;

#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub client: ShortyClient,
    pub short_base: Url,
}
