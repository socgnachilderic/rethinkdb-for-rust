use std::{borrow::Cow, marker::PhantomData};

use futures::{Stream, TryStreamExt};
use ql2::term::TermType;
use serde::{de::DeserializeOwned, Serialize};

use crate::ops::{ReqlOps, ReqlOpsObject};
use crate::{Command, Func};

use super::StaticString;

#[derive(Debug, Clone)]
pub struct MaxBuilder<T>(pub(crate) Command, pub(crate) PhantomData<T>);

#[derive(Debug, Clone, Serialize, Default, PartialEq, PartialOrd)]
#[non_exhaustive]
pub(crate) struct MaxOption {
    pub index: Option<Cow<'static, str>>,
}

impl<T: Unpin + Serialize + DeserializeOwned> MaxBuilder<T> {
    pub(crate) fn new() -> Self {
        Self::constructor(None)
    }

    pub(crate) fn new_by_value(field_name: &str) -> Self {
        let arg = Command::from_json(field_name);
        Self::constructor(Some(arg))
    }

    pub(crate) fn new_by_func(func: Func) -> Self {
        let Func(func) = func;
        Self::constructor(Some(func))
    }

    pub async fn run(self, arg: impl super::run::Arg) -> crate::Result<Option<T>> {
        self.make_query(arg).try_next().await
    }

    pub fn make_query(self, arg: impl super::run::Arg) -> impl Stream<Item = crate::Result<T>> {
        self.get_parent().run::<_, T>(arg)
    }

    pub fn with_index(mut self, index: &'static str) -> Self {
        let index = Some(index.static_string());
        let index = MaxOption { index };

        self.0 = self.0.with_opts(index);

        self
    }

    pub(crate) fn _with_parent(mut self, parent: Command) -> Self {
        self.0 = self.0.with_parent(parent);
        self
    }

    fn constructor(arg: Option<Command>) -> Self {
        let mut command = Command::new(TermType::Max);

        if let Some(arg) = arg {
            command = command.with_arg(arg)
        }

        Self(command, PhantomData)
    }
}

impl<T> ReqlOpsObject<T> for MaxBuilder<T> {}

impl<T> ReqlOps for MaxBuilder<T> {
    fn get_parent(&self) -> Command {
        self.0.clone().into_arg::<()>().into_cmd()
    }
}

impl<T> Into<Command> for MaxBuilder<T> {
    fn into(self) -> Command {
        self.get_parent()
    }
}
