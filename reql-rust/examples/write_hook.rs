use reql_rust::{r, Result, Session};
use reql_rust::prelude::*;

#[tokio::main]
async fn main() -> Result<()> {
    let mut conn = r.connection().connect().await?;
    set_up(&conn).await?;
    conn.use_("marvel").await;

    let result = r
        .table("heroes")
        .set_write_hook(func!(|context, _, _| context.bracket("function")))
        .run(&conn)
        .try_next()
        .await?;
    dbg!(result);

    tear_down(&conn).await?;
    Ok(())
}

async fn set_up(conn: &Session) -> Result<()> {
    r.db_create("marvel").run(conn).try_next().await?;
    r.db("marvel")
        .table_create("heroes")
        .run(conn)
        .try_next()
        .await?;

    Ok(())
}

async fn tear_down(conn: &Session) -> Result<()> {
    r.table_drop("heroes").run(conn).try_next().await?;
    r.db_drop("marvel").run(conn).try_next().await?;

    Ok(())
}
