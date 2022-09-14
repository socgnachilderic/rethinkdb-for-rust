use crate::Command;
use ql2::term::TermType;

pub(crate) fn new(index_name: &str) -> Command {
    let args = Command::from_json(index_name);

    Command::new(TermType::IndexDrop).with_arg(args)
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use crate::prelude::*;
    use crate::types::IndexResponse;
    use crate::{r, Result};

    #[tokio::test]
    async fn test_drop_db() -> Result<()> {
        let table_name = Uuid::new_v4().to_string();
        let index_name = "author";
        let conn = r.connection().connect().await?;
        let table = r.table(table_name.as_str());

        r.table_create(table_name.as_str()).run(&conn).await?;
        table.clone().index_create(index_name).run(&conn).await?;

        let index_dropped: IndexResponse = table
            .index_drop(index_name)
            .run(&conn)
            .await?
            .unwrap()
            .parse()?;

        assert!(index_dropped.dropped > Some(0));

        r.table_drop(table_name.as_str()).run(&conn).await?;
        Ok(())
    }
}