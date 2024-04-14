use axum::extract::FromRef;
use leptos::LeptosOptions;
use tonic::transport::Channel;
use url::Url;

use crate::grpc::shorty_client;

pub type ShortyClient = shorty_client::ShortyClient<Channel>;

#[derive(FromRef, Clone, Debug)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub client: ShortyClient,
    pub short_base: Url,
}
