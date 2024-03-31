use axum::extract::FromRef;
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use leptos::LeptosOptions;
use url::Url;

#[derive(FromRef, Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: Pool<RedisConnectionManager>,
    pub public_base: Url,
}
