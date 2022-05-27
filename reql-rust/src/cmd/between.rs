use std::{borrow::Cow, marker::PhantomData};

use super::StaticString;
use crate::{Command, Result, types::Status, document::Document};
use futures::{Stream, TryStreamExt};
use ql2::term::TermType;
use serde::{de::DeserializeOwned, Serialize};

#[derive(Debug, Clone)]
pub struct BetweenBuilder<T>(
    pub(crate) Command,
    pub(crate) BetweenOption,
    pub(crate) PhantomData<T>
);

#[derive(Debug, Clone, Serialize, Default, Eq, PartialEq, Ord, PartialOrd, Hash)]
#[non_exhaustive]
pub struct BetweenOption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub index: Option<Cow<'static, str>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub left_bound: Option<Status>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub right_bound: Option<Status>,
}

impl<T: Unpin + Serialize + DeserializeOwned> BetweenBuilder<T> {
    pub(crate) fn new(lower_key: impl Serialize, upper_key: impl Serialize) -> Self {
        let min_key = Command::from_json(lower_key);
        let max_key = Command::from_json(upper_key);
        
        let command = Command::new(TermType::Between)
            .with_arg(min_key)
            .with_arg(max_key);

        Self(command, BetweenOption::default(), PhantomData)
    }

    pub async fn run(self, arg: impl super::run::Arg) -> Result<Option<Vec<Document<T>>>> {
        self.make_query(arg)
            .try_next()
            .await
    }

    pub fn make_query(
        self,
        arg: impl super::run::Arg,
    ) -> impl Stream<Item = Result<Vec<Document<T>>>> {
        self.0
            .with_opts(self.1)
            .into_arg::<()>()
            .into_cmd()
            .run::<_, Vec<Document<T>>>(arg)
    }

    pub fn with_index(mut self, index: &'static str) -> Self {
        self.1.index = Some(index.static_string());
        self
    }

    pub fn with_left_bound(mut self, status: Status) -> Self {
        self.1.left_bound = Some(status);
        self
    }

    pub fn with_right_bound(mut self, status: Status) -> Self {
        self.1.right_bound = Some(status);
        self
    }

    #[doc(hidden)]
    pub(crate) fn _with_parent(mut self, parent: Command) -> Self {
        self.0 = self.0.with_parent(parent);
        self
    }
}
