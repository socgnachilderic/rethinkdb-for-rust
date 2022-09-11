#![allow(clippy::wrong_self_convention)]

pub mod cmd;
pub mod connection;
mod constants;
mod err;

pub mod prelude;
mod proto;
#[cfg(test)]
mod spec;
pub mod types;

use futures::Future;
pub use prelude::Func;
use serde::Serialize;
use std::sync::atomic::{AtomicU64, Ordering};

pub use connection::*;
pub use err::*;
pub use proto::Command;

#[doc(hidden)]
pub static VAR_COUNTER: AtomicU64 = AtomicU64::new(1);

#[doc(hidden)]
pub fn var_counter() -> u64 {
    VAR_COUNTER.fetch_add(1, Ordering::SeqCst)
}

// #[cfg(test)]
// fn current_counter() -> u64 {
//     VAR_COUNTER.load(Ordering::SeqCst)
// }

pub type Result<T> = std::result::Result<T, ReqlError>;

#[allow(non_camel_case_types)]
pub struct r;

impl r {
    pub fn connection(self) -> cmd::connect::ConnectionCommand {
        cmd::connect::ConnectionCommand::default()
    }

    pub fn db_create(self, db_name: &str) -> Command {
        cmd::db_create::new(db_name)
    }

    pub fn db_drop(self, db_name: &str) -> Command {
        cmd::db_drop::new(db_name)
    }

    pub fn db_list(self) -> Command {
        cmd::db_list::new()
    }

    pub fn db(self, db_name: &str) -> Command {
        cmd::db::new(db_name)
    }

    pub fn table_create(self, args: impl cmd::table_create::TableCreateArg) -> Command {
        cmd::table_create::new(args)
    }

    pub fn table_drop(self, table_name: &str) -> Command {
        cmd::table_drop::new(table_name)
    }

    pub fn table_list(self) -> Command {
        cmd::table_list::new()
    }

    pub fn table(self, args: impl cmd::table::TableArg) -> Command {
        cmd::table::new(args)
    }

    pub fn map(self, args: impl cmd::map::MapArg) -> Command {
        cmd::map::new(args)
    }

    pub fn order_by(self, args: impl cmd::order_by::OrderByArg) -> Command {
        cmd::order_by::new(args)
    }

    pub fn union(self, args: impl cmd::union::UnionArg) -> Command {
        cmd::union::new(args)
    }

    pub fn reduce(self, args: impl cmd::reduce::ReduceArg) -> Command {
        cmd::reduce::new(args)
    }

    pub fn count(self, args: impl cmd::count::CountArg) -> Command {
        cmd::count::new(args)
    }

    pub fn sum(self, args: impl cmd::sum::SumArg) -> Command {
        cmd::sum::new(args)
    }

    pub fn avg(self, args: impl cmd::avg::AvgArg) -> Command {
        cmd::avg::new(args)
    }

    pub fn min(self, args: impl cmd::min::MinArg) -> Command {
        cmd::min::new(args)
    }

    pub fn max(self, args: impl cmd::max::MaxArg) -> Command {
        cmd::max::new(args)
    }

    pub fn distinct(self, args: impl cmd::distinct::DistinctArg) -> Command {
        cmd::distinct::new(args)
    }

    pub fn contains(self, args: impl cmd::contains::ContainsArg) -> Command {
        cmd::contains::new(args)
    }

    pub fn literal(self, value: impl Serialize) -> Command {
        cmd::literal::new(value)
    }

    pub fn object(self, values: Vec<impl Serialize>) -> Command {
        cmd::object::new(values)
    }

    /* pub fn random(self, arg: impl cmd::random::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn round(self, arg: impl cmd::round::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn ceil(self, arg: impl cmd::ceil::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn floor(self, arg: impl cmd::floor::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn now(self) -> DateTime {
        DateTime::now()
    }

    pub fn time(self, date: Date, timezone: UtcOffset, time: Option<Time>) -> DateTime {
        DateTime::time(date, timezone, time)
    }

    pub fn epoch_time(self, timestamp: i64) -> crate::Result<DateTime> {
        DateTime::epoch_time(timestamp)
    }

    pub fn iso8601(
        self,
        iso_datetime: &str,
        default_timezone: Option<UtcOffset>,
    ) -> crate::Result<DateTime> {
        DateTime::iso8601(iso_datetime, default_timezone)
    }

    pub fn do_(self, func: Func) -> cmd::do_::DoBuilder {
        cmd::do_::DoBuilder::new(func)
    }

    pub fn branch(self, arg: impl cmd::branch::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn range(self, arg: impl cmd::range::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn error(self, arg: impl cmd::error::Arg) -> Command {
        arg.arg().into_cmd()
    }*/

    pub fn expr(self, arg: impl cmd::expr::Arg) -> Command {
        arg.arg().into_cmd()
    }

    /*pub fn js(self, arg: impl cmd::js::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn info(self, arg: impl cmd::info::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn json(self, arg: impl cmd::json::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn http(self, arg: impl cmd::http::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn uuid(self, arg: impl cmd::uuid::Arg) -> Command {
        arg.arg().into_cmd()
    }

    pub fn circle(self, point: &Point, radius: u32) -> cmd::circle::CircleBuilder<Polygon> {
        cmd::circle::CircleBuilder::new(point, radius)
    }

    pub fn circle_unfill(self, point: &Point, radius: u32) -> cmd::circle::CircleBuilder<Line> {
        cmd::circle::CircleBuilder::new(point, radius).with_fill(false)
    }

    pub fn geojson<T>(self, geojson: &GeoJson<T>) -> cmd::geojson::ReqlGeoJson<T>
    where
        T: Unpin + Serialize + DeserializeOwned + Clone,
    {
        cmd::geojson::ReqlGeoJson::new(geojson)
    }

    pub fn line(self, points: &[cmd::point::Point]) -> cmd::line::Line {
        cmd::line::Line::new(points)
    }

    pub fn point(self, longitude: f64, latitude: f64) -> cmd::point::Point {
        cmd::point::Point::new(longitude, latitude)
    }

    pub fn polygon(self, points: &[cmd::point::Point]) -> cmd::polygon::Polygon {
        cmd::polygon::Polygon::new(points)
    }

    pub fn grant(self, username: &str) -> cmd::grant::GrantBuilder {
        cmd::grant::GrantBuilder::new(username)
    }

    pub fn asc(self, arg: impl cmd::asc::Arg) -> cmd::asc::Asc {
        cmd::asc::Asc(arg.arg().into_cmd())
    }

    pub fn desc(self, arg: impl cmd::desc::Arg) -> cmd::desc::Desc {
        cmd::desc::Desc(arg.arg().into_cmd())
    }

    pub fn index(self, arg: impl cmd::index::Arg) -> cmd::index::Index {
        cmd::index::Index(arg.arg().into_cmd())
    } */

    pub fn args<T>(self, arg: T) -> cmd::args::Args<T> {
        cmd::args::Args(arg)
    }

    pub fn min_val() -> Command {
        Command::new(ql2::term::TermType::Minval)
    }

    pub fn max_val() -> Command {
        Command::new(ql2::term::TermType::Maxval)
    }
}

// Helper for making writing examples less verbose
#[doc(hidden)]
pub async fn example<Q, F>(query: impl FnOnce(r, Session) -> F) -> Result<()>
where
    F: Future<Output = Result<()>>,
{
    query(r, r.connection().connect().await?).await
}
