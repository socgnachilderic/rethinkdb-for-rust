use futures::TryStreamExt;
use reql_rust::r;
use serde_json::Value;

#[tokio::test]
async fn table() -> reql_rust::Result<()> {
    tracing_subscriber::fmt::init();
    let conn = r.connect(()).await?;
    let mut query = r.db("rethinkdb").table("users").run(&conn);
    let user: Option<Value> = query.try_next().await?;
    assert!(user.is_some());
    Ok(())
}
