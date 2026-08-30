pub async fn redis_ready(redis_url: &str) -> anyhow::Result<()> {
    let client = redis::Client::open(redis_url)?;
    let mut conn = redis::aio::ConnectionManager::new(client).await?;
    let _: String = redis::cmd("PING").query_async(&mut conn).await?;
    Ok(())
}
