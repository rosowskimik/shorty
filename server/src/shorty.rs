use std::{cell::RefCell, ops::DerefMut, time::SystemTime};

use eyre::Context;
use once_cell::unsync::Lazy;
use rand::distributions::{Alphanumeric, DistString};
use rand_xoshiro::{rand_core::SeedableRng, Xoshiro256PlusPlus};
use redis::AsyncCommands;
use tonic::{Request, Response, Status};
use tracing::{debug, error, instrument, trace};
use url::Url;

use crate::{
    db::{init_db_pool, RedisConn, RedisPool},
    shorty::grpc::{
        shorty_server::{Shorty, ShortyServer},
        SlugMessage, UrlMessage,
    },
};

pub mod grpc {
    tonic::include_proto!("shorty");
}

pub struct AppShorty {
    pool: RedisPool,
}

impl AppShorty {
    pub const SLUG_LEN: usize = 8;
    pub const KEEP_DURATION: u64 = 3600;

    thread_local! {
        static RNG: Lazy<RefCell<Xoshiro256PlusPlus>> = Lazy::new(|| {
            let secs = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs();
            let rand = Xoshiro256PlusPlus::seed_from_u64(secs);
            RefCell::new(rand)
        });
    }

    pub async fn try_new(db: impl AsRef<str>) -> eyre::Result<Self> {
        debug!("Creating new service");

        let pool = init_db_pool(db)
            .await
            .wrap_err("Failed to connect to database")?;

        Ok(Self { pool })
    }

    pub fn grpc_service(self) -> ShortyServer<Self> {
        ShortyServer::new(self)
    }

    fn gen_slug() -> String {
        Self::RNG.with(|v| {
            let mut r = v.borrow_mut();
            Alphanumeric.sample_string(r.deref_mut(), Self::SLUG_LEN)
        })
    }

    async fn get_conn(&self) -> Result<RedisConn, Status> {
        trace!("Getting connection from pool");
        self.pool.get().await.map_err(|e| {
            error!(err = ?e, "Failed to get DB connection from pool");
            Status::unavailable("Pool exhausted")
        })
    }
}

#[tonic::async_trait]
impl Shorty for AppShorty {
    #[instrument(level = "debug", skip_all, fields(req.addr = ?req.remote_addr()))]
    async fn shorten(&self, req: Request<UrlMessage>) -> Result<Response<SlugMessage>, Status> {
        let msg = req.into_inner();

        trace!("Parsing arguments");
        Url::parse(&msg.url).map_err(|_| Status::invalid_argument("Malformed Url"))?;

        let mut conn = self.get_conn().await?;

        let slug = Self::gen_slug();

        trace!(slug, url = msg.url, "Adding new mapping");
        conn.set_ex(slug.clone(), msg.url, Self::KEEP_DURATION)
            .await
            .map_err(|e| {
                error!(slug, err = ?e, "Failed to set new mapping");
                Status::unavailable("Failed to set new mapping")
            })?;

        Ok(Response::new(SlugMessage { slug }))
    }

    #[instrument(level = "debug", skip_all, fields(req.addr = ?req.remote_addr()))]
    async fn get_original(
        &self,
        req: Request<SlugMessage>,
    ) -> Result<Response<UrlMessage>, Status> {
        let msg = req.into_inner();

        let mut conn = self.get_conn().await?;

        trace!(slug = msg.slug, "Looking up mapping");
        let val: Option<String> = conn.get(&msg.slug).await.map_err(|e| {
            error!(slug = msg.slug, err = ?e, "Failed to fetch data");
            Status::unavailable("Failed to fetch data")
        })?;

        if let Some(url) = val {
            Ok(Response::new(UrlMessage { url }))
        } else {
            Err(Status::not_found("Mapping not found"))
        }
    }
}
