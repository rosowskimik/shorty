
use leptos::*;
use url::Url;

#[cfg(feature = "ssr")]
pub mod ssr {
    use std::{
        cell::{LazyCell, RefCell},
        ops::DerefMut,
        time::{Duration,SystemTime},
    };

    use axum::{
        extract::{Path, State},
        http::StatusCode,
        response::Redirect,
    };
    use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256PlusPlus};
    use redis::AsyncCommands;

    use crate::db::RedisPool;

    use super::*;

    pub const SHORT_SLUG_BASE: &'static str = "/s/";
    pub const SHORT_SLUG_LEN: usize = 8;
    pub const SHORT_KEEP_DURATION: u64 = Duration::from_hours(1).as_secs();

    thread_local! {
        pub static RANDOM: LazyCell<RefCell<Xoshiro256PlusPlus>> = LazyCell::new(|| {
            let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let rand = Xoshiro256PlusPlus::seed_from_u64(secs);
            RefCell::new(rand)
        });
    }

    pub async fn short_url_handler(
        State(pool): State<RedisPool>,
        Path(slug): Path<String>,
    ) -> Result<Redirect, (StatusCode, &'static str)> {
        let mut conn = pool.get().await.map_err(|e| {
            tracing::error!("Failed to get DB connection from pool ({})", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
        })?;

        let val: Option<String> = conn.get(&slug).await.map_err(|e| {
            tracing::error!("DB error ({})", e);
            (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong")
        })?;

        if let Some(uri) = val {
            Ok(Redirect::to(&uri))
        } else {
            Err((StatusCode::NOT_FOUND, "Not found"))
        }
    }

    pub fn pool() -> Result<RedisPool, ServerFnError> {
        use_context::<RedisPool>().ok_or_else(|| {
            tracing::error!("DB pool not in context!");
            ServerFnError::ServerError("Something went wrong".to_string())
        })
    }

    pub fn public_base() -> Result<Url, ServerFnError> {
        use_context::<Url>().ok_or_else(|| {
            tracing::error!("Server URL not in context!");
            ServerFnError::ServerError("Something went wrong".to_string())
        })
    }

    pub fn random_slug() -> String {
        use rand::distributions::{Alphanumeric, DistString};

        RANDOM.with(|v| {
            let mut r = v.borrow_mut();
            Alphanumeric.sample_string(r.deref_mut(), SHORT_SLUG_LEN)
        })
    }
}

#[server(ShortenUrl)]
pub async fn shorten_url(url: Url) -> Result<Url, ServerFnError> {
    use redis::AsyncCommands;
    use leptos::server_fn::error::NoCustomError;

    use ssr::{pool, public_base, random_slug, SHORT_KEEP_DURATION, SHORT_SLUG_BASE, SHORT_SLUG_LEN};

    if url.scheme().is_empty() || url.cannot_be_a_base() {
        return Err(ServerFnError::ServerError("Bad URL".to_string()));
    }

    let mut public_base = public_base()?;
    let pool = pool()?;
    let mut conn = pool.get().await.map_err(|e| {
        tracing::error!("Failed to get DB connection from pool ({})", e);
        ServerFnError::<NoCustomError>::ServerError("Something went wrong".to_string())
    })?;

    let tail = random_slug();

    let path = {
        let mut p = String::with_capacity(SHORT_SLUG_BASE.len() + SHORT_SLUG_LEN); 
        p.push_str(SHORT_SLUG_BASE);
        p.push_str(&tail);
        p
    };

    conn.set_ex(tail, url.to_string(), SHORT_KEEP_DURATION).await.map_err(|e| {
        tracing::error!("DB error ({})", e);
        ServerFnError::<NoCustomError>::ServerError("Something went wrong".to_string())
    })?;

    public_base.set_path(&path);
    Ok(public_base)
}
