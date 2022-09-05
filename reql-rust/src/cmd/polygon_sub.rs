use futures::{Stream, TryStreamExt};
use ql2::term::TermType;

use crate::ops::{ReqlOps, ReqlOpsGeometry};
use crate::types::Polygon;
use crate::Command;

pub struct PolygonSubBuilder(pub(crate) Command);

impl PolygonSubBuilder {
    pub(crate) fn new(polygon: &Polygon) -> Self {
        let command = Command::new(TermType::PolygonSub).with_arg(polygon.command.clone().unwrap());

        Self(command)
    }

    pub async fn run(self, arg: impl super::run::Arg) -> crate::Result<Option<Polygon>> {
        self.make_query(arg).try_next().await
    }

    pub fn make_query(
        self,
        arg: impl super::run::Arg,
    ) -> impl Stream<Item = crate::Result<Polygon>> {
        self.get_parent().run::<_, Polygon>(arg)
    }

    pub(crate) fn _with_parent(mut self, parent: Command) -> Self {
        self.0 = self.0.with_parent(parent);
        self
    }
}

impl ReqlOpsGeometry for PolygonSubBuilder {}

impl ReqlOps for PolygonSubBuilder {
    fn get_parent(&self) -> Command {
        self.0.clone().into_arg::<()>().into_cmd()
    }
}

impl Into<Command> for PolygonSubBuilder {
    fn into(self) -> Command {
        self.get_parent()
    }
}
