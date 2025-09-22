use anyhow::Result;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde::{Deserialize, Serialize};

pub struct CacheManager {
    conn: MultiplexedConnection,
}

impl CacheManager {
    pub async fn new(redis_url: &str) -> Result<Self> {
        let client = Client::open(redis_url)?;
        let conn = client.get_multiplexed_tokio_connection().await?;
        Ok(Self { conn })
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(&mut self, key: &str) -> Result<Option<T>> {
        let data: Option<Vec<u8>> = self.conn.get(key).await?;
        match data {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    pub async fn set<T: Serialize>(&mut self, key: &str, value: &T, ttl_secs: u64) -> Result<()> {
        let data = serde_json::to_vec(value)?;
        self.conn.set_ex::<_, _, ()>(key, data, ttl_secs).await?;
        Ok(())
    }

    pub async fn delete(&mut self, key: &str) -> Result<()> {
        self.conn.del::<_, ()>(key).await?;
        Ok(())
    }
}
