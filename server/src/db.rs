use bb8::{Pool, PooledConnection};
use bb8_redis::RedisConnectionManager;
use eyre::bail;
use redis::AsyncCommands;
use tracing::{debug, trace};

pub(crate) type RedisPool = Pool<RedisConnectionManager>;
pub(crate) type RedisConn<'a> = PooledConnection<'a, RedisConnectionManager>;

pub async fn init_db_pool(conn_str: impl AsRef<str>) -> eyre::Result<RedisPool> {
    let addr = conn_str.as_ref();
    debug!(addr, "Connecting to database");

    let manager = RedisConnectionManager::new(addr)?;
    let pool = Pool::builder().build(manager).await?;

    trace!(addr, "Testing connection");
    {
        let mut conn = pool.get().await?;
        conn.set("foo", "bar").await?;
        let result: String = conn.get_del("foo").await?;
        if result != "bar" {
            bail!("Unexpected value");
        }
    }

    Ok(pool)
}
