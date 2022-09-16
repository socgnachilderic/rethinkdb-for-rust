use ql2::term::TermType;
use reql_macros::CommandOptions;
use serde::Serialize;

use crate::arguments::{Args, Unit};
use crate::prelude::Geometry;
use crate::types::GeoSystem;
use crate::Command;

pub(crate) fn new(args: impl DistanceArg) -> Command {
    let (arg1, arg2, opts) = args.into_distance_opts();
    let mut command = Command::new(TermType::Distance).with_arg(arg1);

    if let Some(arg) = arg2 {
        command = command.with_arg(arg)
    }

    command.with_opts(opts)
}

pub trait DistanceArg {
    fn into_distance_opts(self) -> (Command, Option<Command>, DistanceOption);
}

impl<T: Geometry> DistanceArg for T {
    fn into_distance_opts(self) -> (Command, Option<Command>, DistanceOption) {
        (self.into(), None, Default::default())
    }
}

impl<T: Geometry, G: Geometry> DistanceArg for Args<(T, G)> {
    fn into_distance_opts(self) -> (Command, Option<Command>, DistanceOption) {
        (self.0 .0.into(), Some(self.0 .1.into()), Default::default())
    }
}

impl<T: Geometry> DistanceArg for Args<(T, DistanceOption)> {
    fn into_distance_opts(self) -> (Command, Option<Command>, DistanceOption) {
        (self.0 .0.into(), None, self.0 .1)
    }
}

impl<T: Geometry, G: Geometry> DistanceArg for Args<(T, G, DistanceOption)> {
    fn into_distance_opts(self) -> (Command, Option<Command>, DistanceOption) {
        (self.0 .0.into(), Some(self.0 .1.into()), self.0 .2)
    }
}

#[derive(Debug, Clone, Serialize, Default, PartialEq, Eq, PartialOrd, Ord, CommandOptions)]
pub struct DistanceOption {
    /// the reference ellipsoid to use for geographic coordinates.
    /// Possible values are `GeoSystem::WGS84` (the default),
    /// a common standard for Earth’s geometry, or `GeoSystem::UnitSphere`,
    /// a perfect sphere of 1 meter radius.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geo_system: Option<GeoSystem>,
    /// Unit to return the distance in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<Unit>,
}

#[cfg(test)]
mod tests {
    use crate::arguments::Unit;
    use crate::prelude::Converter;
    use crate::{args, r, Result};

    use super::DistanceOption;

    #[tokio::test]
    async fn test_distance_data() -> Result<()> {
        let conn = r.connection().connect().await?;
        let point1 = r.point(-122.423246, 37.779388);
        let point2 = r.point(-117.220406, 32.719464);
        let distance_option = DistanceOption::default().unit(Unit::Kilometer);

        let response: f64 = r
            .distance(args!(point1, point2, distance_option))
            .run(&conn)
            .await?
            .unwrap()
            .parse()?;

        assert!(response == 734.125249602186);

        Ok(())
    }
}
