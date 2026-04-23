use super::redis_key::RedisKey;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, Client};
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;

/// Redis 客户端（使用 MultiplexedConnection）
#[derive(Clone)]
pub struct RedisClient {
    conn: Arc<Mutex<MultiplexedConnection>>,
}

impl RedisClient {
    /// 创建新的 Redis 客户端
    pub async fn new(url: &str) -> redis::RedisResult<Self> {
        let client = Client::open(url)?;
        let conn = client.get_multiplexed_async_connection().await?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// 检查 Redis 连接是否可用
    pub async fn ping(&self) -> redis::RedisResult<String> {
        let mut c = self.conn.lock().await;
        redis::cmd("PING").query_async(&mut *c).await
    }

    /// 设置字符串值
    pub async fn set(&self, k: &str, v: &str) -> redis::RedisResult<()> {
        let mut c = self.conn.lock().await;
        c.set(k, v).await
    }

    /// 获取字符串值
    pub async fn get(&self, k: &str) -> redis::RedisResult<Option<String>> {
        let mut c = self.conn.lock().await;
        c.get(k).await
    }

    /// 自增整数值
    pub async fn incr(&self, k: &str) -> redis::RedisResult<i64> {
        let mut c = self.conn.lock().await;
        c.incr(k, 1).await
    }

    /// 设置字符串值并指定过期时间（秒）
    pub async fn set_ex(&self, k: &str, v: &str, seconds: u64) -> redis::RedisResult<()> {
        let mut c = self.conn.lock().await;
        c.set_ex(k, v, seconds).await
    }

    /// 删除键
    pub async fn del(&self, k: &str) -> redis::RedisResult<()> {
        let mut c = self.conn.lock().await;
        c.del(k).await
    }

    /// 设置键的过期时间（秒）
    pub async fn expire(&self, k: &str, seconds: u64) -> redis::RedisResult<()> {
        let mut c = self.conn.lock().await;
        c.expire(k, seconds as i64).await
    }

    /// 使用 RedisKey 设置 JSON 值
    pub async fn set_key<T: Serialize>(&self, key: &RedisKey, value: &T) -> redis::RedisResult<()> {
        let json = serde_json::to_string(value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "JSON serialization failed",
                e.to_string(),
            ))
        })?;
        let mut c = self.conn.lock().await;
        c.set(key.build(), json).await
    }

    /// 使用 RedisKey 设置 JSON 值并指定过期时间（秒）
    pub async fn set_key_ex<T: Serialize + ?Sized>(
        &self,
        key: &RedisKey,
        value: &T,
        expiration_seconds: u64,
    ) -> redis::RedisResult<()> {
        let json = serde_json::to_string(value).map_err(|e| {
            redis::RedisError::from((
                redis::ErrorKind::TypeError,
                "JSON serialization failed",
                e.to_string(),
            ))
        })?;
        let mut c = self.conn.lock().await;
        c.set_ex(key.build(), json, expiration_seconds).await
    }

    /// 使用 RedisKey 获取字符串值
    pub async fn get_key(&self, key: &RedisKey) -> redis::RedisResult<Option<String>> {
        let mut c = self.conn.lock().await;
        let json: Option<String> = c.get(key.build()).await?;
        Ok(json)
    }

    /// 使用 RedisKey 获取并反序列化 JSON 值
    pub async fn get_key_json<T: for<'de> serde::Deserialize<'de>>(
        &self,
        key: &RedisKey,
    ) -> redis::RedisResult<Option<T>> {
        let mut c = self.conn.lock().await;
        let json: Option<String> = c.get(key.build()).await?;
        match json {
            Some(data) => {
                let value = serde_json::from_str(&data).map_err(|e| {
                    redis::RedisError::from((
                        redis::ErrorKind::TypeError,
                        "JSON deserialization failed",
                        e.to_string(),
                    ))
                })?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }

    /// 使用 RedisKey 删除键
    pub async fn delete_key(&self, key: &RedisKey) -> redis::RedisResult<()> {
        let mut c = self.conn.lock().await;
        c.del(key.build()).await
    }

    /// 使用 RedisKey 检查键是否存在
    pub async fn exists_key(&self, key: &RedisKey) -> redis::RedisResult<bool> {
        let mut c = self.conn.lock().await;
        c.exists(key.build()).await
    }

    /// 使用 RedisKey 设置键的过期时间（秒）
    pub async fn expire_key(&self, key: &RedisKey, seconds: u64) -> redis::RedisResult<()> {
        let mut c = self.conn.lock().await;
        c.expire(key.build(), seconds as i64).await
    }
}
