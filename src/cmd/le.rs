use ql2::term::TermType;
use serde::Serialize;

use crate::{types::AnyParam, Command};

use super::CmdOpts;

pub(crate) fn new(args: impl LeArg) -> Command {
    args.into_le_opts().add_to_cmd(Command::new(TermType::Le))
}

pub trait LeArg {
    fn into_le_opts(self) -> CmdOpts;
}

impl LeArg for AnyParam {
    fn into_le_opts(self) -> CmdOpts {
        CmdOpts::Single(self.into())
    }
}

impl<T: Serialize> LeArg for Vec<T> {
    fn into_le_opts(self) -> CmdOpts {
        let commands = self.iter().map(Command::from_json).collect();

        CmdOpts::Many(commands)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::Converter;
    use crate::spec::{set_up, tear_down, TABLE_NAMES};
    use crate::types::AnyParam;
    use crate::{r, Result};

    #[tokio::test]
    async fn test_le_data() -> Result<()> {
        let (conn, table) = set_up(TABLE_NAMES[0], true).await?;
        let data_obtained: bool = table
            .get(1)
            .g("view")
            .le(AnyParam::new(10))
            .run(&conn)
            .await?
            .unwrap()
            .parse()?;

        assert!(data_obtained);

        tear_down(conn, TABLE_NAMES[0]).await
    }

    #[tokio::test]
    async fn test_le_data_r() -> Result<()> {
        let conn = r.connection().connect().await?;
        let data_obtained: bool = r.le(vec![5, 6, 7]).run(&conn).await?.unwrap().parse()?;

        assert!(data_obtained);

        Ok(())
    }
}
