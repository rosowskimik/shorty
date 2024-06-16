use std::time::Duration;

use bb8::{Pool, PooledConnection, RunError};
use bb8_redis::RedisConnectionManager;
use eyre::bail;
use redis::{AsyncCommands, RedisError};
use tracing::{debug, info, warn};

pub(crate) type RedisPool = Pool<RedisConnectionManager>;
pub(crate) type RedisConn<'a> = PooledConnection<'a, RedisConnectionManager>;

pub struct DbController {
    master_pool: RedisPool,
    replica_pool: Option<RedisPool>,
}

impl DbController {
    pub async fn init(
        master_str: impl AsRef<str>,
        replica_str: Option<impl AsRef<str>>,
    ) -> eyre::Result<Self> {
        let addr = master_str.as_ref();
        info!(addr, "Connecting to database");

        let manager = RedisConnectionManager::new(addr)?;
        let master_pool = Pool::builder().build(manager).await?;

        #[cfg(debug_assertions)]
        {
            debug!(addr, "Testing connection");
            {
                let mut conn = master_pool.get().await?;
                conn.set("foo", "bar").await?;
                let result: String = conn.get_del("foo").await?;
                if result != "bar" {
                    bail!("Unexpected value");
                }
            }
        }

        let replica_pool = if let Some(replica_str) = replica_str {
            let addr = replica_str.as_ref();
            info!(addr, "Connecting to database replica");

            let manager = RedisConnectionManager::new(addr)?;
            Some(
                Pool::builder()
                    .connection_timeout(Duration::from_secs(10))
                    .build(manager)
                    .await?,
            )
        } else {
            None
        };

        Ok(Self {
            master_pool,
            replica_pool,
        })
    }

    pub async fn writer(&self) -> Result<RedisConn, RunError<RedisError>> {
        self.master_pool.get().await
    }

    pub async fn reader(&self) -> Result<RedisConn, RunError<RedisError>> {
        if let Some(ref replica_pool) = self.replica_pool {
            match replica_pool.get().await {
                Err(e) => {
                    warn!(warn = ?e, "Connection from replica pool failed. Attempting with master")
                }
                conn => return conn,
            };
        };
        self.writer().await
    }
}
