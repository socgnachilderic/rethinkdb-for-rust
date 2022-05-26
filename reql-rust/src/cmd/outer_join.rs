use futures::{Stream, TryStreamExt};
use ql2::term::TermType;
use serde::{de::DeserializeOwned, Serialize};

use crate::{types::JoinResponseType, Command, Func};

use super::{run, table::TableBuilder, JoinOps, DocManipulationOps, SuperOps};

#[derive(Debug, Clone)]
pub struct OuterJoinBuilder<A, T>(
    pub(crate) Command, 
    pub(crate) Option<A>, 
    pub(crate) Option<T>
);

impl<A, T> OuterJoinBuilder<A, T>
where
    A: Unpin + Serialize + DeserializeOwned,
    T: Unpin + Serialize + DeserializeOwned,
{
    pub(crate) fn new(other_table: &TableBuilder<A>, func: Func) -> Self {
        let Func(func) = func;
        let command = Command::new(TermType::OuterJoin)
            .with_arg(other_table.0.clone())
            .with_arg(func);

        Self(command, None, None)
    }

    pub async fn run(
        self,
        arg: impl run::Arg,
    ) -> crate::Result<Option<Vec<JoinResponseType<T, A>>>> {
        self.make_query(arg).try_next().await
    }

    pub fn make_query(
        self,
        arg: impl run::Arg,
    ) -> impl Stream<Item = crate::Result<Vec<JoinResponseType<T, A>>>> {
        self.0
            .into_arg::<()>()
            .into_cmd()
            .run::<_, Vec<JoinResponseType<T, A>>>(arg)
    }

    pub(crate) fn _with_parent(mut self, parent: Command) -> Self {
        self.0 = self.0.with_parent(parent);
        self
    }
}

impl<A, T> JoinOps for OuterJoinBuilder<A, T> { }

impl<A, T> DocManipulationOps for OuterJoinBuilder<A, T> {}

impl<A, T> SuperOps for OuterJoinBuilder<A, T> {
    fn get_parent(&self) -> Command {
        self.0.clone()
    }
}
