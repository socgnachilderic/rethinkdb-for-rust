use std::borrow::Cow;
use std::collections::HashMap;

use ql2::term::TermType;
use reql_macros::CommandOptions;
use serde::{Serialize, Serializer};

use crate::arguments::{EmergencyRepair, Replicas};
use crate::Command;

pub(crate) fn new(opts: ReconfigureOption) -> Command {
    Command::new(TermType::Reconfigure).with_opts(opts)
}

#[derive(Debug, Clone, Default, PartialEq, CommandOptions)]
#[non_exhaustive]
pub struct ReconfigureOption {
    /// the number of shards, an integer from 1-64. Required.
    pub shards: Option<u8>,
    /// either an usize or a mapping struct. Required.
    /// - If `replicas` is an usize, it specifies the number of replicas per shard. 
    /// Specifying more replicas than there are servers will return an error.
    /// - If `replicas` is an struct, it specifies key-value pairs of server tags 
    /// and the number of replicas to assign to those servers: 
    /// `{"tag1": 2, "tag2": 4, "tag3": 2, ...}`. 
    /// For more information about server tags, read 
    /// [Administration tools](https://rethinkdb.com/docs/administration-tools/).
    pub replicas: Option<Replicas>,
    /// the generated configuration will not be applied to the table, only returned.
    pub dry_run: Option<bool>,
    /// Used for the Emergency Repair mode. 
    /// See <https://rethinkdb.com/api/python/reconfigure#emergency-repair-mode>
    /// for more information.
    pub emergency_repair: Option<EmergencyRepair>,
}

impl Serialize for ReconfigureOption {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        #[derive(Serialize)]
        struct InnerOptions<'a> {
            #[serde(skip_serializing_if = "Option::is_none")]
            dry_run: Option<bool>,
            #[serde(skip_serializing_if = "Option::is_none")]
            emergency_repair: Option<EmergencyRepair>,
            #[serde(skip_serializing_if = "Option::is_none")]
            shards: Option<u8>,
            #[serde(skip_serializing_if = "Option::is_none")]
            replicas: Option<InnerReplicas<'a>>,
            /// the primary server specified by its server tag. 
            /// Required if `replicas` is an object; the tag must be in the object. 
            /// This must not be specified if `replicas` is an usize.
            #[serde(skip_serializing_if = "Option::is_none")]
            primary_replica_tag: Option<&'a Cow<'static, str>>,
        }

        #[derive(Serialize)]
        #[serde(untagged)]
        enum InnerReplicas<'a> {
            Int(usize),
            Map(&'a HashMap<Cow<'static, str>, usize>),
        }

        let (replicas, primary_replica_tag) = match &self.replicas {
            Some(Replicas::Int(i)) => (Some(InnerReplicas::Int(*i)), None),
            Some(Replicas::Map {
                replicas,
                primary_replica_tag,
            }) => (
                Some(InnerReplicas::Map(replicas)),
                Some(primary_replica_tag),
            ),
            None => (None, None),
        };

        let opts = InnerOptions {
            dry_run: self.dry_run,
            emergency_repair: self.emergency_repair,
            replicas,
            primary_replica_tag,
            shards: self.shards,
        };

        opts.serialize(serializer)
    }
}

#[cfg(test)]
mod tests {
    use crate::arguments::Replicas;
    use crate::prelude::Converter;
    use crate::spec::{set_up, tear_down};
    use crate::types::ReconfigureResponse;
    use crate::Result;

    use super::ReconfigureOption;

    #[tokio::test]
    async fn test_reconfigure_table() -> Result<()> {
        let (conn, table, table_name) = set_up(true).await?;
        let reconfigure_option = ReconfigureOption::default()
            .shards(2)
            .replicas(Replicas::Int(1));
        let response: ReconfigureResponse = table
            .reconfigure(reconfigure_option)
            .run(&conn)
            .await?
            .unwrap()
            .parse()?;

        assert!(response.reconfigured == 1);

        tear_down(conn, &table_name).await
    }
}
