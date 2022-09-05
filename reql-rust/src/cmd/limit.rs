use std::marker::PhantomData;

use futures::{Stream, TryStreamExt};
use ql2::term::TermType;
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::ops::{ReqlOps, ReqlOpsArray, ReqlOpsDocManipulation, ReqlOpsSequence};
use crate::Command;

#[derive(Debug, Clone)]
pub struct LimitBuilder<T>(pub(crate) Command, pub(crate) PhantomData<T>);

impl<T: Unpin + DeserializeOwned> LimitBuilder<T> {
    pub(crate) fn new(step: usize) -> Self {
        let arg = Command::from_json(step);
        let command = Command::new(TermType::Limit).with_arg(arg);

        Self(command, PhantomData)
    }

    pub async fn run(self, arg: impl super::run::Arg) -> crate::Result<Option<T>> {
        self.make_query(arg).try_next().await
    }

    pub fn make_query(self, arg: impl super::run::Arg) -> impl Stream<Item = crate::Result<T>> {
        self.get_parent().run::<_, T>(arg)
    }

    pub(crate) fn _with_parent(mut self, parent: Command) -> Self {
        self.0 = self.0.with_parent(parent);
        self
    }
}

impl<T: Unpin + Serialize + DeserializeOwned> ReqlOpsSequence<T> for LimitBuilder<T> {}
impl<T> ReqlOpsArray for LimitBuilder<T> {}
impl<T> ReqlOpsDocManipulation for LimitBuilder<T> {}

impl<T> ReqlOps for LimitBuilder<T> {
    fn get_parent(&self) -> Command {
        self.0.clone().into_arg::<()>().into_cmd()
    }
}

impl<T> Into<Command> for LimitBuilder<T> {
    fn into(self) -> Command {
        self.get_parent()
    }
}
