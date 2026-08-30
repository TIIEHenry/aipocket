pub async fn redis_ready(redis_url: &str) -> anyhow::Result<()> {
    let probe = async {
        let client = redis::Client::open(redis_url)?;
        let mut conn = redis::aio::ConnectionManager::new(client).await?;
        let _: String = redis::cmd("PING").query_async(&mut conn).await?;
        Ok(())
    };
    tokio::time::timeout(std::time::Duration::from_secs(2), probe)
        .await
        .map_err(|_| anyhow::anyhow!("redis ready timed out"))?
}

#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn redis_ready_rejects_closed_port() {
        let result = super::redis_ready("redis://127.0.0.1:1/").await;
        assert!(result.is_err());
    }
}
