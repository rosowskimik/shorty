use std::env;

#[cfg(not(debug_assertions))]
use anyhow::Context;
use anyhow::{bail, Result};
use bb8::Pool;
use bb8_redis::RedisConnectionManager;
use redis::AsyncCommands;
use tracing::{debug, info};

use crate::env::DB_CONN_ENV;

pub type RedisPool = Pool<RedisConnectionManager>;

pub async fn init_db_pool() -> Result<RedisPool> {
    #[cfg(not(debug_assertions))]
    let conn_str = env::var(DB_CONN_ENV).with_context(|| format!("Database connection string '{}' not set in environment!", DB_CONN_ENV))?;
    #[cfg(debug_assertions)]
    let conn_str = env::var(DB_CONN_ENV).unwrap_or("redis://localhost".to_string());

    info!("Connecting to database '{}'", conn_str);
    let manager = RedisConnectionManager::new(conn_str)?;
    let pool = Pool::builder().build(manager).await?;

    debug!("Testing DB connection");
    {
        let mut conn = pool.get().await?;
        conn.set("foo", "bar").await?;
        let result: String = conn.get_del("foo").await?;
        if result != "bar" {
            bail!("Failed to connect to database");
        }
    };

    Ok(pool)
}
