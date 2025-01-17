pub mod add;
pub mod and;
pub mod append;
pub mod args;
pub mod array;
pub mod asc;
pub mod avg;
pub mod between;
pub mod binary;
pub mod bit_and;
pub mod bit_not;
pub mod bit_or;
pub mod bit_sal;
pub mod bit_sar;
pub mod bit_xor;
pub mod bracket;
pub mod branch;
pub mod ceil;
pub mod change_at;
pub mod changes;
pub mod circle;
pub mod coerce_to;
pub mod concat_map;
pub mod config;
pub mod connect;
pub mod contains;
pub mod count;
pub mod date;
pub mod day;
pub mod day_of_week;
pub mod day_of_year;
pub mod db;
pub mod db_create;
pub mod db_drop;
pub mod db_list;
pub mod default;
pub mod delete;
pub mod delete_at;
pub mod desc;
pub mod difference;
pub mod distance;
pub mod distinct;
pub mod div;
pub mod do_;
pub mod downcase;
pub mod during;
pub mod epoch_time;
pub mod eq;
pub mod eq_join;
pub mod error;
pub mod expr;
pub mod fill;
pub mod filter;
pub mod floor;
pub mod fold;
pub mod for_each;
pub(crate) mod func;
pub mod ge;
pub mod geojson;
pub mod get;
pub mod get_all;
pub mod get_field;
pub mod get_intersecting;
pub mod get_nearest;
pub mod get_write_hook;
pub mod grant;
pub mod group;
pub mod gt;
pub mod has_fields;
pub mod hash_map;
pub mod hours;
pub mod http;
pub mod in_timezone;
pub mod includes;
pub mod index;
pub mod index_create;
pub mod index_drop;
pub mod index_list;
pub mod index_rename;
pub mod index_status;
pub mod index_wait;
pub mod info;
pub mod inner_join;
pub mod insert;
pub mod insert_at;
pub mod intersects;
pub mod is_empty;
pub mod iso8601;
pub mod js;
pub mod json;
pub mod keys;
pub mod le;
pub mod limit;
pub mod line;
pub mod literal;
pub mod lt;
pub mod map;
pub mod match_;
pub mod max;
pub mod merge;
pub mod min;
pub mod minutes;
pub mod month;
pub mod mul;
pub mod ne;
pub mod not;
pub mod now;
pub mod nth;
pub mod object;
pub mod offsets_of;
pub mod or;
pub mod order_by;
pub mod outer_join;
pub mod pluck;
pub mod point;
pub mod polygon;
pub mod polygon_sub;
pub mod prepend;
pub mod random;
pub mod range;
pub mod rebalance;
pub mod reconfigure;
pub mod reduce;
pub mod rem;
pub mod replace;
pub mod round;
pub mod run;
pub mod sample;
pub mod seconds;
pub mod set_difference;
pub mod set_insert;
pub mod set_intersection;
pub mod set_union;
pub mod set_write_hook;
pub mod skip;
pub mod slice;
pub mod splice_at;
pub mod split;
pub mod status;
pub mod sub;
pub mod sum;
pub mod sync;
pub mod table;
pub mod table_create;
pub mod table_drop;
pub mod table_list;
pub mod time;
pub mod time_of_day;
pub mod timezone;
pub mod to_epoch_time;
pub mod to_geojson;
pub mod to_iso8601;
pub mod to_json;
pub mod type_of;
pub mod ungroup;
pub mod union;
pub mod upcase;
pub mod update;
pub mod uuid;
pub mod values;
pub mod wait;
pub mod with_fields;
pub mod without;
pub mod year;
pub mod zip;

use std::ops::{BitAnd, BitOr, BitXor};
use std::str;

use ::time::UtcOffset;
use futures::stream::Stream;
use futures::TryStreamExt;
use serde_json::Value;

use crate::arguments::{Permission, ReconfigureOption};
use crate::{Command, CommandArg, Func, Result};

impl<'a> Command {
    /// Turn a query into a changefeed, an infinite stream of objects
    /// representing changes to the query’s results as they occur.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// stream.changes(()) → stream
    /// stream.changes(options) → stream
    /// ```
    ///
    /// Where:
    /// - options: [ChangesOption](crate::arguments::ChangesOption)
    ///
    /// # Description
    ///
    /// A changefeed may return changes to a table or
    /// an individual document (a “point” changefeed).
    /// Commands such as filter or map may be used before
    /// the changes command to transform or filter the output, and
    /// many commands that operate on sequences can be chained after changes.
    ///
    /// There are currently two states:
    /// - `{state: 'initializing'}` indicates the following documents represent
    /// initial values on the feed rather than changes.
    /// This will be the first document of a feed that returns initial values.
    /// - `{state: 'ready'}` indicates the following documents represent changes.
    /// This will be the first document of a feed that does **not** return initial values;
    /// otherwise, it will indicate the initial values have all been sent.
    ///
    /// ```text
    /// Starting with RethinkDB 2.2, state documents will only be sent
    /// if the include_states option is true, even on point changefeeds.
    /// Initial values will only be sent if include_initial is true.
    /// If include_states is true and include_initial is false,
    /// the first document on the feed will be {'state': 'ready'}.
    /// ```
    ///
    /// If the table becomes unavailable, the changefeed will be disconnected,
    /// and a runtime exception will be thrown by the driver.
    ///
    /// Changefeed notifications take the form of a two-field object:
    ///
    /// ```text
    /// {
    ///     "old_val": <document before change>,
    ///     "new_val": <document after change>
    /// }
    /// ```
    ///
    /// When `include_types` is `true`, there will be three fields:
    ///
    /// ```text
    /// {
    ///     "old_val": <document before change>,
    ///     "new_val": <document after change>,
    ///     "type": <result type>
    /// }
    /// ```
    ///
    /// When a document is deleted, `new_val` will be `None`;
    /// when a document is inserted, `old_val` will be `None`.
    ///
    /// ## Note
    ///
    /// Certain document transformation commands can be chained before changefeeds.
    /// For more information, read the
    /// [discussion of changefeeds](https://rethinkdb.com/docs/changefeeds/python/)
    /// in the “Query language” documentation.
    ///
    /// Changefeeds ignore the `read_mode` flag to `run`, and always behave as
    /// if it is set to `single` (i.e., the values they return are in memory on
    /// the primary replica, but have not necessarily been written to disk yet).
    /// For more details read [Consistency guarantees](https://rethinkdb.com/docs/consistency).
    ///
    /// The server will buffer up to 100,000 elements.
    /// If the buffer limit is hit, early changes will be discarded,
    /// and the client will receive an object of the form
    /// `{"error": "Changefeed cache over array size limit, skipped X elements."}`
    /// where X is the number of elements skipped.
    ///
    /// Commands that operate on streams (such as [filter](Self::filter)
    /// or [map](Self::map)) can usually be chained after `changes`.
    /// However, since the stream produced by `changes` has no ending,
    /// commands that need to consume the entire stream before returning
    /// (such as [reduce](Self::reduce) or [count](Self::count)) cannot.
    ///
    /// ## Examples
    ///
    /// Subscribe to the changes on a table.
    ///
    /// ```
    /// use futures::TryStreamExt;
    /// use neor::arguments::ChangesOption;
    /// use neor::types::ChangesResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::Value;
    ///
    /// async fn example() -> Result<()> {
    ///     let session = r.connection().connect().await?;
    ///     let mut connection = session.connection()?;
    ///     let mut response = Vec::new();
    ///     let conn = connection.clone();
    ///     let changes_options = ChangesOption::default()
    ///         .include_initial(true)
    ///         .include_states(true)
    ///         .include_types(true);
    ///
    ///     let mut query = r.table("simbad").changes(()).build_query(conn);
    ///
    ///     while let Some(value) = query.try_next().await? {
    ///         response = value.parse::<Vec<ChangesResponse<Value>>>()?;
    ///         
    ///         connection.close(false).await?;
    ///         break;
    ///     }
    ///
    ///     assert!(response.len() > 0);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn changes(&self, args: impl changes::ChangesArg) -> Self {
        changes::new(args).with_parent(self)
    }

    /// Create a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// db.table_create(table_name) → response
    /// db.table_create(args!(table_name, options)) → response
    /// ```
    ///
    /// Where:
    /// - table_name: `impl Into<String>` | [Command](crate::Command)
    /// - options: [TableCreateOption](crate::arguments::TableCreateOption)
    /// - response: [DbResponse](crate::types::DbResponse)
    ///
    /// # Description
    ///
    /// A RethinkDB table is a collection of JSON documents.
    ///
    /// If a table with the same name already exists,
    /// the command throws `ReqlOpFailedError`.
    ///
    /// ```text
    /// Note: Only alphanumeric characters and underscores are valid for the table name.
    ///
    /// Invoking table_create without specifying a database using db creates a table in
    /// the database specified in connect, or test if no database was specified.
    /// ```
    ///
    /// The [data type](https://rethinkdb.com/docs/data-types/) of a primary key is usually a string
    /// (like a UUID) or a number, but it can also be a time, binary object, boolean or an array.
    /// Data types can be mixed in the primary key field, but all values must be unique. Using an array
    /// as a primary key causes the primary key to behave like a compound index; read the documentation on
    /// [compound secondary indexes](https://rethinkdb.com/docs/secondary-indexes/python/#compound-indexes)
    /// for more information, as it applies to primary keys as well.
    /// (Note that the primary index still only covers a single field,
    /// while compound secondary indexes can cover multiple fields in a single index.)
    /// Primary keys cannot be objects.
    ///
    /// Tables will be available for writing when the command returns.
    ///
    /// ## Examples
    ///
    /// Create a table named ‘simbad’ with the default settings.
    ///
    /// ```
    /// use neor::types::DbResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: DbResponse = r.db("test")
    ///         .table_create("simbad")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.tables_created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Create a table named ‘simbad’ using the field ‘name’ as primary key.
    ///
    /// ```
    /// use neor::arguments::TableCreateOption;
    /// use neor::types::DbResponse;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let table_create_option = TableCreateOption::default().primary_key("name");
    ///     let conn = r.connection().connect().await?;
    ///     let response: DbResponse = r.db("test")
    ///         .table_create(args!("simbad", table_create_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.tables_created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Create a table set up for two shards and three replicas per shard.
    /// This requires three available servers.
    ///
    /// ```
    /// use neor::arguments::{Replicas, TableCreateOption};
    /// use neor::types::DbResponse;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let table_create_option = TableCreateOption::default()
    ///         .shards(2)
    ///         .replicas(Replicas::Int(3));
    ///     let conn = r.connection().connect().await?;
    ///     let response: DbResponse = r.db("test")
    ///         .table_create(args!("simbad", table_create_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.tables_created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Read [Sharding and replication](https://rethinkdb.com/docs/sharding-and-replication/)
    /// for a complete discussion of the subject, including advanced topics.
    ///
    /// # Related commands
    /// - [table_drop](Self::table_drop)
    /// - [table_list](Self::table_list)
    pub fn table_create(&self, args: impl table_create::TableCreateArg) -> Self {
        table_create::new(args).with_parent(self)
    }

    /// Drop a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// db.table_drop(table_name) → response
    /// ```
    ///
    /// Where:
    /// - table_name: `impl Into<String>` | [Command](crate::Command)
    /// - response: [DbResponse](crate::types::DbResponse)
    ///
    /// # Description
    ///
    /// The table and all its data will be deleted.
    ///
    /// If the given table does not exist in the database,
    /// the command throws `ReqlRuntimeError`.
    ///
    /// ## Examples
    ///
    /// Drop a table named ‘simbad’.
    ///
    /// ```
    /// use neor::types::DbResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: DbResponse = r.db("test")
    ///         .table_drop("simbad")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.tables_dropped > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [table_create](Self::table_create)
    /// - [table_list](Self::table_list)
    pub fn table_drop(&self, table_name: impl Into<CommandArg>) -> Self {
        table_drop::new(table_name).with_parent(self)
    }

    /// List all table names in a database.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// db.table_list() → response
    /// ```
    ///
    /// Where:
    /// - response: Vec<String>
    ///
    /// ## Examples
    ///
    /// List all tables of the ‘test’ database.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<String> = r.db("test")
    ///         .table_list()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.len() > 0);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [table_create](Self::table_create)
    /// - [table_drop](Self::table_drop)
    pub fn table_list(&self) -> Self {
        table_list::new().with_parent(self)
    }

    /// Return all documents in a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// db.table(name) → table
    /// db.table(args!(name, options)) → table
    /// r.table(name) → table
    /// r.table(args!(name, options)) → table
    /// ```
    ///
    /// Where:
    /// - name: `impl Into<String>` | [Command](crate::Command)
    /// - options: [TableOption](crate::arguments::TableOption)
    ///
    /// # Description
    ///
    /// Other commands may be chained after `table` to return a subset of documents
    /// (such as [get](crate::Command::get) and [filter](crate::Command::filter))
    /// or perform further processing.
    ///
    /// ## Examples
    ///
    /// Return all documents in the table ‘simbad’ of the default database.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("simbad").run(&conn).await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return all documents in the table ‘simbad’ of the database ‘heroes’.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.db("heroes").table("simbad").run(&conn).await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Allow potentially out-of-date data in exchange for faster reads.
    ///
    /// ```
    /// use neor::arguments::{ReadMode, TableOption};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let table_option = TableOption::default().read_mode(ReadMode::Outdated);
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.db("heroes").table(args!("simbad", table_option)).run(&conn).await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [filter](Self::filter)
    /// - [get](Self::get)
    pub fn table(&self, args: impl table::TableArg) -> Self {
        table::new(args).with_parent(self)
    }

    /// Create a new secondary index on a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.index_create(index_name) → response
    /// table.index_create(args!(index_create, func)) → response
    /// table.index_create(args!(index_create, options)) → response
    /// table.index_create(args!(index_create, func, options)) → response
    /// ```
    ///
    /// Where:
    /// - index_name: `impl Into<String>` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [IndexCreateOption](crate::arguments::IndexCreateOption)
    /// - response: [IndexResponse](crate::types::IndexResponse)
    ///
    /// # Description
    ///
    /// Secondary indexes improve the speed of many read queries at the slight
    /// cost of increased storage space and decreased write performance.
    /// For more information about secondary indexes,  read the article
    /// “[Using secondary indexes in RethinkDB](https://rethinkdb.com/docs/secondary-indexes/python/).”
    ///
    /// RethinkDB supports different types of secondary indexes:
    /// - **Simple indexes** based on the value of a single field.
    /// - **Compound indexes** based on multiple fields.
    /// - **Multi indexes** based on arrays of values.
    /// - **Geospatial indexes** based on indexes of geometry objects,
    /// created when the geo optional argument is true.
    /// - Indexes based on **arbitrary expressions**.
    ///
    /// The `index_function` can be an anonymous function or a binary
    /// representation obtained from the `function` field of [index_status](Self::index_status).
    /// The function must be deterministic, and so cannot use a subquery or the `r.js` command.
    ///
    /// If successful, `create_index` will return an object of the form `{"created": 1}`.
    /// If an index by that name already exists on the table, a `ReqlRuntimeError` will be thrown.
    ///
    /// ```text
    /// that an index may not be immediately available after creation.
    /// If your application needs to use indexes immediately after creation,
    /// use the index_wait command to ensure the indexes are ready before use.
    /// ```
    ///
    /// ## Examples
    ///
    /// Create a simple index based on the field `post_id`.
    ///
    /// ```
    /// use neor::types::IndexResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexResponse = r.table("comments")
    ///         .index_create("post_id")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Create a simple index based on the nested field `author > name`.
    ///
    /// ```
    /// use neor::types::IndexResponse;
    /// use neor::{args, func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexResponse = r.table("comments")
    ///         .index_create(args!("author_name", func!(|comment| comment.g("author").g("name"))))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Create a geospatial index based on the field `location`.
    ///
    /// ```
    /// use neor::arguments::IndexCreateOption;
    /// use neor::types::IndexResponse;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let index_create_option = IndexCreateOption::default().geo(true);
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexResponse = r.table("places")
    ///         .index_create(args!("location", index_create_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// A geospatial index field should contain only geometry objects.
    /// It will work with geometry ReQL terms
    /// ([get_intersecting](Self::get_intersecting) and [get_nearest](Self::get_nearest))
    /// as well as index-specific terms  ([index_status](Self::index_status),
    /// [index_wait](Self::index_wait), [index_drop](Self::index_drop) and [index_list](Self::index_list)).
    /// Using terms that rely on non-geometric ordering such as [get_all](Self::get_all),
    /// [order_by](Self::order_by) and [between](Self::between) will result in an error.
    ///
    /// ## Examples
    ///
    /// Create a compound index based on the fields `post_id` and `date`.
    ///
    /// ```
    /// use neor::arguments::IndexCreateOption;
    /// use neor::types::IndexResponse;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let index_create_option = IndexCreateOption::default().geo(true);
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexResponse = r.table("comments")
    ///         .index_create(args!("post_and_date", index_create_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.created > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [index_wait](Self::index_wait)
    /// - [index_status](Self::index_status)
    /// - [index_list](Self::index_list)
    /// - [index_drop](Self::index_drop)
    pub fn index_create(&self, args: impl index_create::IndexCreateArg) -> Self {
        index_create::new(args).with_parent(self)
    }

    /// Delete a previously created secondary index of this table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.index_drop(index_name) → response
    /// ```
    ///
    /// Where:
    /// - index_name: `impl Into<String>` | [Command](crate::Command)
    /// - response: [IndexResponse](crate::types::IndexResponse)
    ///
    /// ## Examples
    ///
    /// Drop a secondary index named ‘code_name’.
    ///
    /// ```
    /// use neor::types::IndexResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexResponse = r.table("comments")
    ///         .index_drop("code_name")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.dropped > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [index_create](Self::index_create)
    /// - [index_list](Self::index_list)
    pub fn index_drop(&self, index_name: impl Into<CommandArg>) -> Self {
        index_drop::new(index_name).with_parent(self)
    }

    /// List all the secondary indexes of this table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.index_list() → response
    /// ```
    ///
    /// Where:
    /// - response: Vec<String>
    ///
    /// ## Examples
    ///
    /// List the available secondary indexes for this table.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<String> = r.table("comments")
    ///         .index_list()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.len() > 0);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [index_create](Self::index_create)
    /// - [index_drop](Self::index_drop)
    pub fn index_list(&self) -> Self {
        index_list::new().with_parent(self)
    }

    /// Rename an existing secondary index on a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.index_rename(args!(old_index_name, new_index_name)) → response
    /// table.index_rename(args!(old_index_name, new_index_name, options)) → response
    /// ```
    ///
    /// Where:
    /// - old_index_name, new_index_name: `impl Into<String>` | [Command](crate::Command)
    /// - options: [IndexRenameOption](crate::arguments::IndexRenameOption)
    /// - response: [IndexResponse](crate::types::IndexResponse)
    ///
    /// # Description
    ///
    /// If the optional argument `overwrite` is specified as `true`,
    /// a previously existing index with the new name will be deleted
    /// and the index will be renamed. If `overwrite` is `false` (the default)
    /// an error will be raised if the new index name already exists.
    ///
    /// The return value on success will be an object of the format `{'renamed': 1}`,
    /// or `{'renamed': 0}` if the old and new names are the same.
    ///
    /// An error will be raised if the old index name does not exist,
    /// if the new index name is already in use and `overwrite` is `false`,
    /// or if either the old or new index name are the same as the primary key field name.
    ///
    /// ## Examples
    ///
    /// Rename an index on the comments table.
    ///
    /// ```
    /// use neor::types::IndexResponse;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexResponse = r.table("comments")
    ///         .index_rename(args!("post_id", "message_id"))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.renamed > Some(0));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [index_create](Self::index_create)
    /// - [index_status](Self::index_status)
    /// - [index_list](Self::index_list)
    /// - [index_drop](Self::index_drop)
    pub fn index_rename(&self, args: impl index_rename::IndexRenameArg) -> Self {
        index_rename::new(args).with_parent(self)
    }

    /// Get the status of the specified indexes on this table,
    /// or the status of all indexes on this table if no indexes are specified.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.index_status(()) → response
    /// table.index_status(index) → response
    /// table.index_status(args!(indexes)) → response
    /// ```
    ///
    /// Where:
    /// - index: `&str`
    /// - indexes: `impl IntoIterator<Item = impl Into<String>>` |
    /// `impl IntoIterator<Item = Command>`
    /// - response: Vec<[IndexStatusResponse](crate::types::IndexStatusResponse)>
    ///
    /// # Description
    ///
    /// The `multi` field will be `true` or `false` depending on whether
    /// this index was created as a multi index
    /// (see [index_create](Self::index_create) for details).
    /// The `outdated` field will be true if the index is outdated in
    /// the current version of RethinkDB and needs to be rebuilt.
    /// The `progress` field is a float between `0` and `1`,
    /// indicating how far along the server is in constructing indexes after
    /// the most recent change to the table that would affect them.
    /// (`0` indicates no such indexes have been constructed; `1` indicates all of them have.)
    ///
    /// The `function` field is a binary object containing an opaque representation
    /// of the secondary index (including the `multi` argument if specified).
    /// It can be passed as the second argument to [index_create](Self::index_create) to create
    /// a new index with the same function; see `index_create` for more information.
    ///
    /// ## Examples
    ///
    /// Get the status of all the indexes on `test`:
    ///
    /// ```
    /// use neor::types::IndexStatusResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<IndexStatusResponse> = r.table("test")
    ///         .index_status(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.len() > 0);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get the status of the `timestamp` index:
    ///
    /// ```
    /// use neor::types::IndexStatusResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<IndexStatusResponse> = r.table("test")
    ///         .index_status("timestamp")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.len() > 0);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Save the binary representation of the index:
    ///
    /// ```
    /// use neor::types::IndexStatusResponse;
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("test")
    ///         .index_status("timestamp")
    ///         .nth(0)
    ///         .g("function")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [index_wait](Self::index_wait)
    pub fn index_status(&self, args: impl index_status::IndexStatusArg) -> Self {
        index_status::new(args).with_parent(self)
    }

    /// Wait for the specified indexes on this table to be ready,
    /// or for all indexes on this table to be ready if no indexes are specified.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.index_wait(()) → response
    /// table.index_wait(index) → response
    /// table.index_status(args!(indexes)) → response
    /// ```
    ///
    /// Where:
    /// - index: `&str`
    /// - indexes: `impl IntoIterator<Item = impl Into<String>>` |
    /// `impl IntoIterator<Item = Command>`
    /// - response: [IndexStatusResponse](crate::types::IndexStatusResponse)
    ///
    /// ## Examples
    ///
    /// Wait for all indexes on the table `test` to be ready:
    ///
    /// ```
    /// use neor::types::IndexStatusResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexStatusResponse = r.table("test")
    ///         .index_wait(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.ready);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Wait for the index `timestamp` to be ready:
    ///
    /// ```
    /// use neor::types::IndexStatusResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: IndexStatusResponse = r.table("test")
    ///         .index_wait("timestamp")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.ready);
    ///     assert_eq!(response.index, "author");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [index_status](Self::index_status)
    pub fn index_wait(&self, args: impl index_wait::IndexWaitArg) -> Self {
        index_wait::new(args).with_parent(self)
    }

    /// Sets the write hook on a table or overwrites it if one already exists.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.set_write_hook(params) → response
    /// ```
    ///
    /// Where:
    /// - params: `Option::<T>::None` | [Func](crate::Func) |
    /// [Binary](crate::types::Binary) | [Command](crate::Command)
    /// - response: [SetWriteHookResponse](crate::types::SetWriteHookResponse)
    ///
    /// # Description
    ///
    /// The `function` can be an anonymous function with the signature
    /// `(context: object, old_val: object, new_val: object) -> object`
    /// or a binary representation obtained from the `function` field
    /// of [get_write_hook](Self::get_write_hook).
    /// The function must be deterministic,
    /// and so cannot use a subquery or the `r.js` command.
    ///
    /// The first argument, `context`,
    /// is a ReQL object containing the following properties:
    /// - `primary_key`: primary key of the document being deleted, inserted, or modified
    /// - `timestamp`: a ReQL `time` object representing the current query execution time
    ///
    /// Whenever a write operation on the table inserts, deletes or modifies a given document,
    /// the write hook function will be called with the context parameter, the old value of
    /// the document (or `null` on inserts) and the new value of the document (or `null` on deletes).
    /// It then returns the value that should actually be inserted and/or replaced instead of `newVal`.
    /// It can also return `r.error(...)` to abort the write.
    ///
    /// For simplicity, the write hook function is allowed
    /// to return `null` exactly if and only if `newVal` is `null`.
    /// This is just a safeguard to ensure you don’t accidentally
    /// turn an insert/update into a deletion, or a deletion into an update.
    ///
    /// ## Examples
    ///
    /// Create a write hook
    ///
    /// ```
    /// use neor::types::SetWriteHookResponse;
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: SetWriteHookResponse = r.table("comments")
    ///         .set_write_hook(func!(|_, _, new_val| new_val))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response.created, Some(1));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete the write hook associated with the comments table.
    ///
    /// ```
    /// use neor::types::SetWriteHookResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: SetWriteHookResponse = r.table("comments")
    ///         .set_write_hook(Option::<u8>::None)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response.deleted, Some(1));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get_write_hook](Self::get_write_hook)
    pub fn set_write_hook(&self, args: impl Into<CommandArg>) -> Self {
        set_write_hook::new(args).with_parent(self)
    }

    /// Gets the write hook of this table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.get_write_hook() → response
    /// ```
    ///
    /// Where:
    /// - response: [GetWriteHookResponse](crate::types::GetWriteHookResponse)
    ///
    /// # Description
    ///
    /// If a write hook exists, the result is an object of the following form:
    ///
    /// ```text
    /// {
    ///     "function": <binary>,
    ///     "query": "setWriteHook(function(_var1, _var2, _var3) { return ...; })",
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get the write hook for the `comments` table.
    ///
    /// ```
    /// use neor::types::GetWriteHookResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: GetWriteHookResponse = r.table("comments")
    ///         .get_write_hook()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(
    ///         response.query,
    ///         "setWriteHook(function(var1, var2, var3) { return var3; })"
    ///     );
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [set_write_hook](Self::set_write_hook)
    pub fn get_write_hook(&self) -> Self {
        get_write_hook::new().with_parent(self)
    }

    /// Insert documents into a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.insert(object) → response
    /// table.insert(args!(object, options)) → response
    /// ```
    ///
    /// Where:
    /// - object: `impl Serialize` | [Command](crate::Command)
    /// - options: [InsertOption](crate::arguments::InsertOption)
    /// - response: [MutationResponse](crate::types::MutationResponse)
    ///
    /// # Description
    ///
    /// Accepts a single document or an array of documents.
    ///
    /// ## Examples
    ///
    /// Insert a document into the table `posts`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .insert(json!({
    ///             "id": 1,
    ///             "title": "Lorem ipsum",
    ///             "content": "Dolor sit amet",
    ///         }))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.inserted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Insert a document without a defined primary key into
    /// the table `posts` where the primary key is `id`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .insert(json!({
    ///             "title": "Lorem ipsum",
    ///             "content": "Dolor sit amet",
    ///         }))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.inserted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Insert multiple documents into the table `users`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("users")
    ///         .insert(json!([
    ///             {"id": "malika", "email": "malika@rethinkdb.com"},
    ///             {"id": "moussa", "email": "moussa@rethinkdb.com"}
    ///         ]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.inserted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Insert a document into the table `users`,
    /// replacing the document if the document already exists.
    ///
    /// ```
    /// use neor::arguments::{Conflict, InsertOption};
    /// use neor::types::MutationResponse;
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let insert_option = InsertOption::default().conflict(Conflict::Replace);
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("users")
    ///         .insert(args!(
    ///             json!({"id": "malika", "email": "malika@rethinkdb.com"}),
    ///             insert_option
    ///         ))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.inserted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Copy the documents from `posts` to `posts_backup`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts_backup")
    ///         .insert(r.table("posts"))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.inserted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get back a copy of the inserted document (with its generated primary key).
    ///
    /// ```
    /// use neor::arguments::{InsertOption, ReturnChanges};
    /// use neor::types::MutationResponse;
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let insert_option = InsertOption::default().return_changes(ReturnChanges::Bool(true));
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .insert(args!(
    ///             json!({"title": "Lorem ipsum", "content": "Dolor sit amet"}),
    ///             insert_option
    ///         ))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.inserted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [update](Self::update)
    /// - [replace](Self::replace)
    /// - [delete](Self::delete)
    pub fn insert(&self, args: impl insert::InsertArg) -> Self {
        insert::new(args).with_parent(self)
    }

    /// Update JSON documents in a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.replace(object) → response
    /// table.replace(func) → response
    /// table.replace(args!(object, options)) → response
    /// table.replace(args!(func, options)) → response
    /// ```
    ///
    /// Where:
    /// - object: `impl Serialize` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [ReplaceOption](crate::arguments::ReplaceOption)
    /// - response: [MutationResponse](crate::types::MutationResponse)
    ///
    /// # Description
    ///
    /// Accepts a JSON document, a ReQL expression, or a combination of the two.
    ///
    /// ## Examples
    ///
    /// Update the status of the post with `id` of `1` to `published`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(json!({"status": "published"}))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Update the status of all posts to `published`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .update(json!({"status": "published"}))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 100);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Update the status of all the posts written by Moussa.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .filter(json!({"author": "Moussa"}))
    ///         .update(json!({"status": "published"}))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 5);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ```text
    /// Note that `filter`, `get_all` and similar operations
    /// do **not** execute in an atomic fashion with `update`.
    /// Read [Consistency guarantees](https://rethinkdb.com/docs/consistency) for more details.
    /// Also, see the example for conditional updates
    /// below for a solution using `branch` in an `update` clause.
    /// ```
    ///
    /// ## Examples
    ///
    /// Increment the field `view` of the post with `id` of `1`.
    /// This query will throw an error if the field `views` doesn’t exist.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{func, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(func!(|post| post.g("views") + r.expr(1)))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Increment the field view of the post with id of 1.
    /// If the field views does not exist, it will be set to 0.
    ///
    /// ```
    /// use std::ops::Add;
    ///
    /// use neor::types::MutationResponse;
    /// use neor::{func, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(func!(|post| post.g("views").add(r.expr(1)).default(0)))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Perform a conditional update.
    ///
    /// If the post has more than 100 views,
    /// set the `type` of a post to `hot`,
    /// else set it to `normal`.
    ///
    /// ```
    /// use std::ops::Add;
    ///
    /// use neor::types::MutationResponse;
    /// use neor::{args, func, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(func!(|post| r.branch(
    ///             post.g("views").gt(100),
    ///             args!(
    ///                 json!({"type": "hot"}),
    ///                 json!({"type": "normal"})
    ///             )
    ///         )))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Update the field `num_comments` with the result of a sub-query.
    /// Because this update is not atomic, you must pass the `non_atomic` flag.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::arguments::UpdateOption;
    /// use neor::types::MutationResponse;
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let update_option = UpdateOption::default().non_atomic(true);
    ///     let conn = r.connection().connect().await?;
    ///     let mut comments_filtered = HashMap::new();
    ///     comments_filtered.insert(
    ///         "num_comments",
    ///         r.table("comments")
    ///             .filter(json!({"id_post": 1}))
    ///             .count(())
    ///     );
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(args!(r.hash_map(comments_filtered), update_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// If you forget to specify the `non_atomic` flag,
    /// you will get a `ReqlRuntimeError`:
    ///
    /// ```text
    /// ReqlRuntimeError: Could not prove function deterministic.
    /// Maybe you want to use the non_atomic flag?
    /// ```
    ///
    /// ## Examples
    ///
    /// Update the field `num_comments` with a random value between 0 and 100.
    /// This update cannot be proven deterministic because of `r.js` (and in fact is not),
    /// so you must pass the `non_atomic` flag.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::arguments::UpdateOption;
    /// use neor::types::MutationResponse;
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let update_option = UpdateOption::default().non_atomic(true);
    ///     let conn = r.connection().connect().await?;
    ///     let mut comments_filtered = HashMap::new();
    ///     comments_filtered.insert(
    ///         "num_comments",
    ///         r.js("Math.floor(Math.random()*100)")
    ///     );
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(args!(r.hash_map(comments_filtered), update_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Update the status of the post with `id` of `1` using soft durability.
    ///
    /// ```
    /// use neor::arguments::{Durability, UpdateOption};
    /// use neor::types::MutationResponse;
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let update_option = UpdateOption::default().durability(Durability::Soft);
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .update(args!(json!({"status": "published"}), update_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [insert](Self::insert)
    /// - [replace](Self::replace)
    /// - [delete](Self::delete)
    pub fn update(&self, args: impl update::UpdateArg) -> Self {
        update::new(args).with_parent(self)
    }

    /// Replace documents in a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.replace(object) → response
    /// table.replace(func) → response
    /// table.replace(args!(object, options)) → response
    /// table.replace(args!(func, options)) → response
    /// ```
    ///
    /// Where:
    /// - object: `impl Serialize` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [ReplaceOption](crate::arguments::ReplaceOption)
    /// - response: [MutationResponse](crate::types::MutationResponse)
    ///
    /// # Description
    ///
    /// Accepts a JSON document or a ReQL expression, and replaces
    /// the original document with the new one.
    /// The new document must have the same primary key as the original document.
    ///
    /// The `replace` command can be used to both insert and delete documents.
    /// If the “replaced” document has a primary key that doesn’t exist in the table,
    /// the document will be inserted; if an existing document is replaced with `None`,
    /// the document will be deleted.
    /// Since `update` and `replace` operations are performed atomically,
    /// this allows atomic inserts and deletes as well.
    ///
    /// ## Examples
    ///
    /// Replace the document with the primary key `1`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .replace(json!({
    ///             "id": 1,
    ///             "title": "Lorem ipsum",
    ///             "content": "Aleas jacta est",
    ///             "status": "draft"
    ///         }))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Remove the field `status` from all posts.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .replace(func!(|post| post.without("status")))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 5);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Remove all the fields that are not `id`, `title` or `content`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .replace(func!(|post| post.pluck(["id", "title", "content"])))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 5);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Replace the document with the primary key `1` using soft durability.
    ///
    /// ```
    /// use neor::arguments::{Durability, ReplaceOption};
    /// use neor::types::MutationResponse;
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let replace_option = ReplaceOption::default().durability(Durability::Soft);
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("posts")
    ///         .get(1)
    ///         .replace(args!(json!({
    ///             "id": 1,
    ///             "title": "Lorem ipsum",
    ///             "content": "Aleas jacta est",
    ///             "status": "draft"
    ///         }), replace_option))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.replaced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Replace the document with the primary key `1` and return the
    /// values of the document before and after the replace operation.
    ///
    /// ```
    /// use neor::arguments::{ReturnChanges, ReplaceOption};
    /// use neor::{args, r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let replace_option = ReplaceOption::default().return_changes(ReturnChanges::Bool(true));
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .get(1)
    ///         .replace(args!(json!({
    ///             "id": 1,
    ///             "title": "Lorem ipsum",
    ///             "content": "Aleas jacta est",
    ///             "status": "draft"
    ///         }), replace_option))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [insert](Self::insert)
    /// - [update](Self::update)
    /// - [delete](Self::delete)
    pub fn replace(&self, args: impl replace::ReplaceArg) -> Self {
        replace::new(args).with_parent(self)
    }

    /// Delete one or more documents from a table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.delete(()) → response
    /// table.delete(options) → response
    /// ```
    ///
    /// Where:
    /// - options: [DeleteOption](crate::arguments::DeleteOption)
    /// - response: [MutationResponse](crate::types::MutationResponse)
    ///
    /// ## Examples
    ///
    /// Delete a single document from the table `comments`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("comments")
    ///         .get("7eab9e63-73f1-4f33-8ce4-95cbea626f59")
    ///         .delete(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.deleted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete all documents from the table comments.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("comments")
    ///         .delete(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.deleted == 100);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete all comments where the field `id_post` is `3`.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("comments")
    ///         .filter(json!({"id_post": 3}))
    ///         .delete(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.deleted == 5);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete a single document from the table `comments` and return its value.
    ///
    /// ```
    /// use neor::arguments::{ReturnChanges, DeleteOption};
    /// use neor::types::MutationResponse;
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let expected_data = json!({
    ///         "id": "7eab9e63-73f1-4f33-8ce4-95cbea626f59",
    ///         "author": "Moussa",
    ///         "comment": "Great post",
    ///         "id_post": 3
    ///     });
    ///     let delete_option = DeleteOption::default().return_changes(ReturnChanges::Bool(true));
    ///     let conn = r.connection().connect().await?;
    ///     let response: MutationResponse = r.table("comments")
    ///         .get("7eab9e63-73f1-4f33-8ce4-95cbea626f59")
    ///         .delete(delete_option)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let old_val = response
    ///         .clone()
    ///         .changes
    ///         .unwrap()
    ///         .first()
    ///         .unwrap()
    ///         .to_owned()
    ///         .old_val;
    ///
    ///     assert!(response.deleted == 1);
    ///     assert_eq!(old_val, Some(expected_data));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete all documents from the table `comments` without
    /// waiting for the operation to be flushed to disk.
    ///
    /// ```
    /// use neor::arguments::{Durability, DeleteOption};
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let delete_option = DeleteOption::default().durability(Durability::Soft);
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("comments")
    ///         .delete(delete_option)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [insert](Self::insert)
    /// - [update](Self::update)
    /// - [replace](Self::replace)
    pub fn delete(&self, args: impl delete::DeleteArg) -> Self {
        delete::new(args).with_parent(self)
    }

    /// `sync` ensures that writes on a given
    /// table are written to permanent storage.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.sync() → response
    /// ```
    ///
    /// Where:
    /// - response: [SyncResponse](crate::types::SyncResponse)
    ///
    /// # Description
    ///
    /// Queries that specify soft durability (`durability=Durability::Soft`)
    /// do not give such guarantees, so `sync` can be used to ensure the state of these queries.
    /// A call to `sync` does not return until all previous writes to the table are persisted.
    ///
    /// ## Examples
    ///
    /// After having updated multiple heroes with soft durability,
    /// we now want to wait until these changes are persisted.
    ///
    /// ```
    /// use neor::types::SyncResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: SyncResponse = r.table("simbad")
    ///         .sync()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.synced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [noreply_wait](crate::connection::Session::noreply_wait)
    pub fn sync(&self) -> Self {
        sync::new().with_parent(self)
    }

    /// Get a document by primary key.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.get(keys) → singleRowSelection
    /// ```
    ///
    /// Where:
    /// - keys: `impl Into<String>` | `i8, u8, ..., usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If no document exists with that primary key, `get` will return `None`.
    ///
    /// ## Examples
    ///
    /// Find a document by UUID.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .get("a9849eef-7176-4411-935b-79a6e3c56a74")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Find a document and merge another document with it.
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("heroes")
    ///         .get(3)
    ///         .merge(json!({
    ///             "powers": ["invisibility", "speed"]
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get_all](Self::get_all)
    /// - [between](Self::between)
    /// - [filter](Self::filter)
    pub fn get(&self, args: impl Into<CommandArg>) -> Self {
        get::new(args).with_parent(self)
    }

    /// Get all documents where the given value
    /// matches the value of the requested index.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.get_all(keys) → selection
    /// table.get_all(args!(keys, options)) → selection
    /// ```
    ///
    /// Where:
    /// - keys: `impl IntoIterator<Item = impl Into<String>>` | [Command](crate::Command)
    /// - options: [GetAllOption](crate::arguments::GetAllOption)
    ///
    /// ## Examples
    ///
    /// Secondary index keys are not guaranteed to be unique so we cannot
    /// query via [get](Self::get) when using a secondary index.
    ///
    /// ```
    /// use neor::arguments::GetAllOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let get_all_option = GetAllOption::default().index("code_name");
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .get_all(args!(["man_of_steel"], get_all_option))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Without an index argument, we default to the primary index.
    /// While `get` will either return the document or `None` when no document
    /// with such a primary key value exists, this will return either a one or zero length stream.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("dc")
    ///         .get_all(["superman"])
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// You can get multiple documents in a single call to get_all.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("dc")
    ///         .get_all(["superman", "ant man"])
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Note
    ///
    /// ```text
    /// `get_all` does not perform any de-duplication.
    /// If you pass the same key more than once, the same document will be returned multiple times.
    /// ```
    ///
    /// # Related commands
    /// - [get](Self::get)
    /// - [between](Self::between)
    /// - [filter](Self::filter)
    pub fn get_all(&self, values: impl get_all::GetAllArg) -> Self {
        get_all::new(values).with_parent(self)
    }

    /// Get all documents between two keys.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.between(args!(lower_key, upper_key)) → table_slice
    /// table.between(args!(lower_key, upper_key, options)) → table_slice
    /// ```
    ///
    /// Where:
    /// - lower_key, upper_key: `impl Into<Serialize>` | [Command](crate::Command)
    /// - options: [BetweenOption](crate::arguments::BetweenOption)
    ///
    /// # Description
    ///
    /// You may also use the special constants `r::min_val()` and `r::max_val()` for boundaries,
    /// which represent “less than any index key” and “more than any index key” respectively.
    /// For instance, if you use `r::min_val()` as the lower key, then `between` will return
    /// all documents whose primary keys (or indexes) are less than the specified upper key.
    ///
    /// If you use arrays as indexes (compound indexes),
    /// they will be sorted using
    /// [lexicographical order](https://en.wikipedia.org/wiki/Lexicographical_order).
    /// Take the following range as an example:
    ///
    /// ```text
    /// [[1, "c"] ... [5, "e"]]
    /// ```
    ///
    /// This range includes all compound keys:
    /// - whose first item is 1 and second item is equal or greater than “c”;
    /// - whose first item is between 1 and 5,
    /// **regardless of the value of the second item**;
    /// - whose first item is 5 and second item is less than or equal to “e”.
    ///
    /// ## Examples
    ///
    /// Find all users with primary key >= 10 and < 20 (a normal half-open interval).
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .between(args!(10, 20))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Find all users with primary key >= 10 and <= 20 (an interval closed on both sides).
    ///
    /// ```
    /// use neor::arguments::{BetweenOption, Status};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let between_option = BetweenOption::default().right_bound(Status::Closed);
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .between(args!(10, 20, between_option))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Find all users with primary key < 20.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .between(args!(r::min_val(), 20))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Find all users with primary key > 10.
    ///
    /// ```
    /// use neor::arguments::{BetweenOption, Status};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let between_option = BetweenOption::default().right_bound(Status::Open);
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .between(args!(10, r::max_val(), between_option))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Between can be used on secondary indexes too.
    /// Just pass an optional index argument giving the secondary index to query.
    ///
    /// ```
    /// use neor::arguments::BetweenOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let between_option = BetweenOption::default().index("code_name");
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("dc")
    ///         .between(args!("dark_knight", "man_of_steel", between_option))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get all users whose full name is between “John Smith” and “Wade Welles.”
    ///
    /// ```
    /// use neor::arguments::BetweenOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let between_option = BetweenOption::default().index("full_name");
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("dc")
    ///         .between(args!(["Smith", "John"], ["Welles", "Wade"], between_option))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get](Self::get)
    /// - [get_all](Self::get_all)
    /// - [filter](Self::filter)
    pub fn between(&self, args: impl between::BetweenArg) -> Self {
        between::new(args).with_parent(self)
    }

    /// Return all the elements in a sequence for which the given predicate is true.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// selection.filter(func) → selection
    /// selection.filter(predicate) → selection
    /// selection.filter(args!(func, options)) → selection
    /// selection.filter(args!(predicate, options)) → selection
    /// ```
    ///
    /// Where:
    /// - predicate: `impl Serialize` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [FilterOption](crate::arguments::FilterOption)
    ///
    /// # Description
    ///
    /// Return all the elements in a sequence for which the given predicate is true.
    /// The return value of `filter` will be the same as the input (sequence, stream, or array).
    /// Documents can be filtered in a variety of ways—ranges, nested values, boolean conditions,
    /// and the results of anonymous functions.
    ///
    /// By default, `filter` will silently skip documents with missing fields:
    /// if the predicate tries to access a field that doesn’t exist
    /// (for instance, the predicate `{"age": 30}` applied to a document with no `age` field),
    /// that document will not be returned in the result set, and no error will be generated.
    /// This behavior can be changed with the default optional argument
    /// [FilterOption](crate::arguments::FilterOption).
    ///
    /// ## Note
    ///
    /// `filter` does not use secondary indexes.
    /// For retrieving documents via secondary indexes, consider
    /// [get_all](Self::get_all), [between](Self::between) and [eq_join](Self::eq_join).
    ///
    /// # Basic predicates
    ///
    /// ## Examples
    ///
    /// Get all users who are 30 years old.
    ///
    /// ```
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(json!({"age": 30}))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The predicate `{"age": 30}` selects documents in the `users` table
    /// with an `age` field whose value is 30. Documents with an `age` field
    /// set to any other value or with no `age` field present are skipped.
    ///
    /// While the `{"field": value}` style of predicate is useful for exact matches,
    /// a more general way to write a predicate is to use an anonymous function that
    /// returns `true` or `false`.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("age").eq(30)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// In this case, the function returns `true` if the field `age` is equal to 30.
    ///
    /// Predicates to filter are evaluated on the server, and must use ReQL expressions.
    /// You cannot use standard Java comparison operators such as `==`, `<` / `>` and `||` / `&&`.
    ///
    /// ## Examples
    ///
    /// Get all users who are more than 18 years old.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("age").gt(18)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get all users who are less than 18 years old and more than 13 years old.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("age").lt(18).and(
    ///             user.g("age").gt(13)
    ///         )))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get all users who are more than 18 years old or have their parental consent.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user
    ///             .g("age")
    ///             .ge(18)
    ///             .or(user.g("hasParentalConsent"))
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Get all users who are more than 18 years old or have their parental consent.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user
    ///             .g("age")
    ///             .ge(18)
    ///             .or(user.g("hasParentalConsent"))
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # More complex predicates
    ///
    /// ## Examples
    ///
    /// Retrieve all users who subscribed between January
    /// 1st, 2012 (included) and January 1st, 2013 (excluded).
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    /// use time::macros::{date, offset};
    /// use time::UtcOffset;
    ///
    /// async fn example() -> Result<()> {
    ///     let timezone = UtcOffset::UTC;
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("subscription_date").during(args!(
    ///             r.time(args!(date!(2012 - 1 - 1), timezone)),
    ///             r.time(args!(date!(2015 - 1 - 1), timezone))
    ///         ))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve all users who have a gmail account (whose field email ends with @gmail.com).
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("email").match_("@gmail.com$")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Filter based on the presence of a value in an array.
    ///
    /// Given this schema for the `users` table.
    ///
    /// ```text
    /// {
    ///     "name": String,
    ///     "places_visited": Vec<String>,
    /// }
    /// ```
    ///
    /// Retrieve all users whose field `places_visited` contains `Cameroon`.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("places_visited").contains("Cameroon")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Filter based on nested fields.
    ///
    /// Given this schema for the `users` table.
    ///
    /// ```text
    /// {
    ///     "id": String,
    ///     "name": {
    ///         "first": String,
    ///         "middle": String,
    ///         "last": String,
    ///     },
    /// }
    /// ```
    ///
    /// Retrieve all users named “Moussa Ibrahim
    /// (first name “Moussa”, last name “Ibrahim”),
    /// with any middle name.
    ///
    /// ```
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(json!({
    ///             "name": {
    ///                 "first": "Moussa",
    ///                 "last": "Ibrahim",
    ///             }
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// If you want an exact match for a field that is an object,
    /// you will have to use anonymous functions.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user
    ///             .g("name")
    ///             .g("first")
    ///             .eq("Moussa")
    ///             .and(user.g("name").g("last").eq("Ibrahim"))
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user
    ///             .g("name")
    ///             .eq(r.expr(json!({
    ///                 "first": "Moussa",
    ///                 "last": "Ibrahim",
    ///             })))
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get](Self::get)
    /// - [get_all](Self::get_all)
    /// - [between](Self::between)
    pub fn filter(&self, args: impl filter::FilterArg) -> Self {
        filter::new(args).with_parent(self)
    }

    /// Returns an inner join of two sequences.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.inner_join(other_sequence, func) → response
    /// ```
    ///
    /// Where:
    /// - other_sequence: [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - response: [Vec<JoinResponse<Left, Right>>](crate::types::JoinResponse)
    ///
    /// # Description
    ///
    /// The returned sequence represents an intersection of the left-hand sequence
    /// and the right-hand sequence: each row of the left-hand sequence will be
    /// compared with each row of the right-hand sequence to find all pairs of rows
    /// which satisfy the predicate. In most cases, you will want to follow the join
    /// with [zip](Self::zip) to combine the left and right results.
    ///
    /// ```text
    /// Note that `inner_join` is slower and much less efficient than using `concat_map`
    /// with `get_all`. You should avoid using `inner_join` in commands when possible.
    /// ```
    ///
    /// ## Examples
    ///
    /// Return a list of all matchups between Marvel and DC heroes
    /// in which the DC hero could beat the Marvel hero in a fight.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .inner_join(
    ///             r.table("dc"),
    ///             func!(|marvel, dc| marvel.g("strength").lt(dc.g("strength")))
    ///         )
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// (Compare this to an [outer_join](Self::outer_join) with
    /// the same inputs and predicate, which would return a list
    /// of **all** Marvel heroes along with any DC heroes with a higher strength.)
    ///
    /// # Related commands
    /// - [eq_join](Self::eq_join)
    /// - [outer_join](Self::outer_join)
    /// - [zip](Self::zip)
    pub fn inner_join(&self, other_sequence: Command, func: Func) -> Self {
        inner_join::new(other_sequence, func).with_parent(self)
    }

    /// Returns a left outer join of two sequences.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.outer_join(other_sequence, func) → response
    /// ```
    ///
    /// Where:
    /// - other_sequence: [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - response: [Vec<JoinResponse<Left, Right>>](crate::types::JoinResponse)
    ///
    /// # Description
    ///
    /// The returned sequence represents a union of the left-hand sequence and the
    /// right-hand sequence: all documents in the left-hand sequence will be returned,
    /// each matched with a document in the right-hand sequence if one satisfies the
    /// predicate condition. In most cases, you will want to follow the join with
    /// [zip](Self::zip) to combine the left and right results.
    ///
    /// ```text
    /// Note that `outer_join` is slower and much less efficient than using `concat_map`
    /// with `get_all`. You should avoid using `outer_join` in commands when possible.
    /// ```
    ///
    /// ## Examples
    ///
    /// Return a list of all Marvel heroes, paired with
    /// any DC heroes who could beat them in a fight.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .outer_join(
    ///             r.table("dc"),
    ///             func!(|marvel, dc| marvel.g("strength").lt(dc.g("strength")))
    ///         )
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// (Compare this to an [inner_join](Self::inner_join) with
    /// the same inputs and predicate, which would return a list
    /// only of the matchups in which the DC hero has the higher strength.)
    ///
    /// # Related commands
    /// - [eq_join](Self::eq_join)
    /// - [inner_join](Self::inner_join)
    /// - [zip](Self::zip)
    pub fn outer_join(&self, other_sequence: Command, func: Func) -> Self {
        outer_join::new(other_sequence, func).with_parent(self)
    }

    /// Join tables using a field or function on the left-hand sequence
    /// matching primary keys or secondary indexes on the right-hand table.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.eq_join(args!(left_field, right_table)) → response
    /// sequence.eq_join(args!(func, right_table)) → response
    /// sequence.eq_join(args!(left_field, right_table, options)) → response
    /// sequence.eq_join(args!(func, right_table, options)) → response
    /// ```
    ///
    /// Where:
    /// - left_field: `impl Into<String>` | [Command](crate::Command)
    /// - right_table: [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [EqJoinOption](crate::arguments::EqJoinOption)
    /// - response: [Vec<JoinResponse<Left, Right>>](crate::types::JoinResponse)
    ///
    /// # Description
    ///
    /// `eq_join` is more efficient than other ReQL join types, and operates much faster.
    /// Documents in the result set consist of pairs of left-hand and right-hand documents,
    /// matched when the field on the left-hand side exists and is non-null and an entry
    /// with that field’s value exists in the specified index on the right-hand side.
    ///
    /// The result set of `eq_join` is a stream or array of
    /// [JoinResponse<LeftDocument, RightDocument>](crate::types::JoinResponse).
    /// Each object in the returned set will be an object of the form
    /// `{ left: <LeftDocument>, right: <RightDocument> }`, where the values of `left` and
    /// `right` will be the joined documents.
    /// Use the [zip](Self::zip) command to merge the `left` and `right` fields together.
    ///
    /// The results from `eq_join` are, by default, not ordered.
    ///
    /// Suppose the players table contains these documents:
    ///
    /// ```text
    /// [
    ///     { "id": 1, "player": "Moussa", "game_id": 1 },
    ///     { "id": 2, "player": "Fatima", "game_id": 3 },
    ///     { "id": 3, "player": "Abessolo", "game_id": 2 },
    ///     { "id": 4, "player": "Kamga", "game_id": 2 },
    ///     { "id": 5, "player": "Malika", "game_id": 1 },
    ///     { "id": 6, "player": "Ibrahim", "game_id": 3 }
    /// ]
    /// ```
    ///
    /// The games table contains these documents:
    ///
    /// [
    ///     { "id": 1, "field": "Aurion" },
    ///     { "id": 2, "field": "Adventures of Nyangi" },
    ///     { "id": 3, "field": "Gazkar" }
    /// ]
    ///
    /// ## Examples
    ///
    /// Match players with the games they’ve played against one another.
    ///
    /// Join these tables using `game_id` on the player table and `id` on the games table:
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .eq_join(args!("game_id", r.table("games")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// For more information, see
    /// [Table joins in RethinkDB](https://rethinkdb.com/docs/table-joins/).
    ///
    /// ## Examples
    ///
    /// Use a secondary index on the right table rather than the primary key.
    /// If players have a secondary index on their cities, we can get a list
    /// of arenas with players in the same area.
    ///
    /// ```
    /// use neor::arguments::EqJoinOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .eq_join(args!(
    ///             "city_id",
    ///             r.table("arenas"),
    ///             EqJoinOption::default().index("city_id")
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Use a function instead of a field to join on a more complicated expression.
    /// Suppose the players have lists of favorite games ranked in order in a field
    /// such as "favorites": [3, 2, 1]. Get a list of players and their top favorite:
    ///
    /// ```
    /// use neor::arguments::EqJoinOption;
    /// use neor::{args, func, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .eq_join(args!(
    ///             func!(|player| player.g("favorites").nth(0)),
    ///             r.table("games")
    ///         ))
    ///         .without(json!([
    ///             { "left": ["favorites", "game_id", "id"] },
    ///             { "right": "id" }
    ///         ]))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [inner_join](Self::inner_join)
    /// - [outer_join](Self::outer_join)
    /// - [without](Self::without)
    /// - [zip](Self::zip)
    pub fn eq_join(&self, args: impl eq_join::EqJoinArg) -> Self {
        eq_join::new(args).with_parent(self)
    }

    /// Used to ‘zip’ up the result of a join by merging the ‘right’
    /// fields into ‘left’ fields of each member of the sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// stream.zip() → stream
    /// ```
    ///
    /// ## Examples
    ///
    /// ‘zips up’ the sequence by merging the left and right fields produced by a join.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .eq_join(args!("user_id", r.table("users")))
    ///         .zip()
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq_join](Self::eq_join)
    /// - [inner_join](Self::inner_join)
    /// - [outer_join](Self::outer_join)
    pub fn zip(&self) -> Self {
        zip::new().with_parent(self)
    }

    /// Transform each element of one or more sequences
    /// by applying a mapping function to them.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.map(func) → stream
    /// sequence.map(sequence, func) → stream
    /// sequence.map(sequences, func) → stream
    /// r.map(sequence, func) → stream
    /// r.map(sequence, sequence, func) → stream
    /// r.map(sequence, sequences, func) → stream
    /// ```
    ///
    /// Where:
    /// - func: [Func](crate::Func)
    /// - sequence: [Command](crate::Command)
    /// - sequences: `impl IntoIterator<Item = Command>`
    ///
    /// # Description
    ///
    /// If `map` is run with two or more sequences, it will iterate
    /// for as many items as there are in the shortest sequence.
    ///
    /// Note that `map` can only be applied to sequences, not single values.
    /// If you wish to apply a function to a single value/selection (including an array),
    /// use the [do_](Self::do_) command.
    ///
    /// ## Examples
    ///
    /// Return the first five squares.
    ///
    /// ```
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<u8> = r.expr([1, 2, 3, 4, 5])
    ///         .map(func!(|val| val.clone() * val))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response, [1, 4, 9, 16, 25]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Sum the elements of three sequences.
    ///
    /// ```
    /// use neor::{args, func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let sequence1 = r.expr([100, 200, 300, 400]);
    ///     let sequence2 = r.expr([10, 20, 30, 40]);
    ///     let sequence3 = r.expr([1, 2, 3, 4]);
    ///
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<u32> = r.map(sequence1, args!(
    ///             [sequence2, sequence3],
    ///             func!(|val1, val2, val3| val1 + val2 + val3)
    ///         ))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response, [111, 222, 333, 444]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Rename a field when retrieving documents
    /// using `map` and [merge](Self::merge).
    ///
    /// This example renames the field `id` to `user_id`
    /// when retrieving documents from the table `users`.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .map(func!(|doc| {
    ///             let mut user = HashMap::new();
    ///             user.insert("user_id", doc.g("id"));
    ///             
    ///             doc.merge(r.hash_map(user)).without("id")
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Assign every superhero an archenemy.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("heroes")
    ///         .map(args!(r.table("villains"), func!(|hero, villain| {
    ///             let mut villain_obj = HashMap::new();
    ///             villain_obj.insert("villain", villain);
    ///
    ///             hero.merge(r.hash_map(villain_obj))
    ///         })))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [concat_map](Self::concat_map)
    /// - [reduce](Self::reduce)
    /// - [do_](Self::do_)
    pub fn map(&self, args: impl map::MapArg) -> Self {
        map::new(args).with_parent(self)
    }

    /// Plucks one or more attributes from a sequence of objects, filtering
    /// out any objects in the sequence that do not have the specified fields.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.with_fields(selector) → stream
    /// ```
    ///
    /// Where:
    /// - selector: `impl Into<String>` |
    /// `impl IntoIterator<Item = impl Into<String>>` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// Functionally, this is identical to [has_fields](Self::has_fields)
    /// followed by [pluck](Self::pluck) on a sequence.
    ///
    /// ## Examples
    ///
    /// Get a list of users and their posts, excluding any users who have not made any posts.
    ///
    /// Existing table structure:
    ///
    /// ```text
    /// [
    ///     { "id": 1, "user": "bob", "email": "bob@foo.com", "posts": [ 1, 4, 5 ] },
    ///     { "id": 2, "user": "george", "email": "george@foo.com" },
    ///     { "id": 3, "user": "jane", "email": "jane@foo.com", "posts": [ 2, 3, 6 ] }
    /// ]
    /// ```
    ///
    /// Command and output:
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .with_fields(["id", "user", "posts"])
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///
    ///     // [
    ///     //      { "id": 1, "user": "bob", "posts": [ 1, 4, 5 ] },
    ///     //      { "id": 3, "user": "jane", "posts": [ 2, 3, 6 ] }
    ///     // ]
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [has_fields](Self::has_fields)
    /// - [pluck](Self::pluck)
    /// - [without](Self::without)
    pub fn with_fields(&self, fields: impl Into<CommandArg>) -> Self {
        with_fields::new(fields).with_parent(self)
    }

    /// Concatenate one or more elements into
    /// a single sequence using a mapping function.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// stream.concat_map(func) → stream
    /// ```
    ///
    /// Where:
    /// - func: [Func](crate::Func)
    ///
    /// # Description
    ///
    /// `concat_map` works in a similar fashion to [map](Self::map),
    /// applying the given function to each element in a sequence,
    /// but it will always return a single sequence.
    /// If the mapping function returns a sequence,
    /// `map` would produce a sequence of sequences:
    ///
    /// ```ignore
    /// r.expr([1, 2, 3]).map(func!(|x| r.array(x.clone(), x * 2))).run(&conn)
    /// ```
    ///
    /// Result :
    ///
    /// ```text
    /// [[1, 2], [2, 4], [3, 6]]
    /// ```
    ///
    /// Whereas `concat_map` with the same mapping
    /// function would merge those sequences into one:
    ///
    /// ```
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<u8> = r
    ///         .expr([1, 2, 3])
    ///         .concat_map(func!(|x| r.array([x.clone(), x * 2])))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///     
    ///     assert!(response == vec![1, 2, 2, 4, 3, 6]);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The return value, array or stream, will be the same type as the input.
    ///
    /// ## Examples
    ///
    /// Construct a sequence of all monsters defeated by Marvel heroes.
    /// The field “defeatedMonsters” is an array of one or more monster names.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .concat_map(func!(|hero| hero.g("defeatedMonsters")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    pub fn concat_map(&self, func: Func) -> Command {
        concat_map::new(func).with_parent(self)
    }

    /// Sort the sequence by document values of the given key(s).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.order_by(index) → table_slice
    /// table.order_by(args!(predicate, index)) → table_slice
    /// sequence.order_by(predicate) → array
    /// sequence.order_by(predicates) → array
    /// ```
    ///
    /// Where:
    /// - index: [index](crate::r::index)
    /// - predicate: `impl Into<String>` | [Func](crate::Func) | [Command](crate::Command)
    /// - predicates: `[predicate; N]`
    ///
    /// # Description
    ///
    /// To specify the ordering, wrap the attribute with
    /// either `r.asc` or `r.desc` (defaults to ascending).
    ///
    /// ## Note
    ///
    /// RethinkDB uses byte-wise ordering for `order_by` and does not support
    /// Unicode collations; non-ASCII characters will be sorted by UTF-8 codepoint.
    /// For more information on RethinkDB’s sorting order, read the section in
    /// [ReQL data types](https://rethinkdb.com/docs/data-types/#sorting-order).
    ///
    ///
    /// Sorting without an index requires the server to hold the sequence in memory,
    /// and is limited to 100,000 documents
    /// (or the setting of the `array_limit` option for [run](Self::run)).
    /// Sorting with an index can be done on arbitrarily large tables,
    /// or after a [between](Self::between) command using the same index.
    /// This applies to both secondary indexes and the primary key
    /// (e.g., `r.index("id")`).
    ///
    /// Sorting functions passed to `order_by` must be deterministic.
    /// You cannot, for instance, order rows using the [random](crate::r::random) command.
    /// Using a non-deterministic function with `order_by` will raise a `ReqlQueryLogicError`.
    ///
    /// ## Examples
    ///
    /// Order all the posts using the index `date`.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .order_by(r.index("date"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The index must either be the primary key or have
    /// been previously created with [index_create](Self::index_create).
    ///
    /// ```ignore
    /// r.table("posts").index_create("date").run(&conn)
    /// ```
    ///
    /// ## Examples
    ///
    /// Order a sequence without an index.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .get(1)
    ///         .g("comments")
    ///         .order_by("date")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// You can also select a descending ordering:
    ///
    /// ```ignore
    /// r.table("posts").get(1).g("comments").order_by(r.desc("date")).run(&conn)
    /// ```
    ///
    /// If you’re doing ad-hoc analysis and know your table won’t have more then 100,000 elements
    /// (or you’ve changed the setting of the `array_limit` option for [run](Self::run))
    /// you can run `order_by` without an index:
    ///
    /// ```ignore
    /// r.table("small_table").order_by("date").run(&conn)
    /// ```
    ///
    /// ## Examples
    ///
    /// You can efficiently order using multiple fields by using a
    /// [compound index](https://rethinkdb.com/docs/secondary-indexes/python/).
    ///
    /// Order by date and title.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .order_by(r.index("date_and_title"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The index must either be the primary key or have been
    /// previously created with [index_create](Self::index_create).
    ///
    /// ```ignore
    /// r.table("posts").index_create(
    ///     "date_and_title",
    ///     func!(|post| r.array([post.g("date"), post.g("title")]))
    /// ).run(&conn)
    /// ```
    ///
    /// **Note:** You cannot specify multiple orders in a compound index.
    /// See [issue #2306](https://github.com/rethinkdb/rethinkdb/issues/2306)
    /// to track progress.
    ///
    /// ## Examples
    ///
    /// If you have a sequence with fewer documents than the `array_limit`,
    /// you can order it by multiple fields without an index.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("small_table")
    ///         .order_by(args!([r.expr("date"), r.desc("title")]))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Notice that an index ordering always has highest precedence.
    /// The following query orders posts by date, and if multiple posts
    /// were published on the same date, they will be ordered by title.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .order_by(args!("title", r.index("date")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// You can use
    /// [nested field](https://rethinkdb.com/docs/cookbook/python/#filtering-based-on-nested-fields)
    /// syntax to sort on fields from subdocuments.
    /// (You can also create indexes on nested fields using this syntax with `index_create`.)
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .order_by(func!(|user| user.g("group").g("id")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// You can efficiently order data on arbitrary expressions using indexes.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .order_by(r.index("votes"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The index must have been previously created with index_create.
    ///
    /// ```ignore
    /// r.table("posts").index_create("votes", func!(|post| post.g("upvotes") - post.g("downvotes")))
    /// ```
    ///
    /// ## Examples
    ///
    /// If you have a sequence with fewer documents than the `array_limit`,
    /// you can order it with an arbitrary function directly.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("small_table")
    ///         .order_by(func!(|doc| doc.g("upvotes") - doc.g("downvotes")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// You can also select a descending ordering:
    ///
    /// ```ignore
    /// r.table("small_table").order_by(r.desc(func!(|doc| doc.g("upvotes") - doc.g("downvotes"))))
    /// ```
    ///
    /// # Related commands
    /// - [skip](Self::skip)
    /// - [limit](Self::limit)
    /// - [slice](Self::slice)
    pub fn order_by(&self, args: impl order_by::OrderByArg) -> Self {
        order_by::new(args).with_parent(self)
    }

    /// Skip a number of elements from the head of the sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.skip(n) → stream
    /// ```
    ///
    /// Where:
    /// - n: `u8, u16, ..., usize`
    ///
    /// ## Examples
    ///
    /// Only so many can fit in our Pantheon of heroes.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .order_by(r.index("age"))
    ///         .skip(10)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [order_by](Self::order_by)
    /// - [limit](Self::limit)
    /// - [slice](Self::slice)
    /// - [nth](Self::nth)
    pub fn skip(&self, n: impl Into<CommandArg>) -> Self {
        skip::new(n).with_parent(self)
    }

    /// End the sequence after the given numbers of elements.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.limit(number) → stream
    /// ```
    ///
    /// Where:
    /// - number: `u8, u16, ..., usize`
    ///
    /// ## Examples
    ///
    /// Only so many can fit in our Pantheon of heroes.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .order_by(r.index("age"))
    ///         .limit(10)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [order_by](Self::order_by)
    /// - [skip](Self::skip)
    /// - [slice](Self::slice)
    /// - [nth](Self::nth)
    pub fn limit(&self, number: impl Into<CommandArg>) -> Self {
        limit::new(number).with_parent(self)
    }

    /// Return the elements of a sequence within the specified range.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// selection.slice(start_offset) → selection
    /// selection.slice(args!(start_offset, options)) → selection
    /// selection.slice(args!(start_offset, end_offset)) → selection
    /// selection.slice(args!(start_offset, end_offset, options)) → selection
    /// ```
    ///
    /// Where:
    /// - start_offset, end_offset: `i8, u8, ..., isize, usize` |
    /// [Command](crate::Command)
    /// - options: [SliceOption](crate::arguments::SliceOption)
    ///
    /// # Description
    ///
    /// `slice` returns the range between `start_offset` and `end_offset`.
    /// If only `start_offset` is specified, `slice` returns the range from
    /// that index to the end of the sequence. Specify `left_bound` or `right_bound`
    /// option as `Status::Open` or `Status::Closed` to indicate whether
    /// to include that endpoint of the range by default:
    /// `Status::Closed` returns that endpoint, while `Status::Open` does not.
    /// By default, left_bound is `Status::Closed` and right_bound is `Status::Open`, so
    /// the range `(10,13)` will return the tenth, eleventh and twelfth elements in the sequence.
    ///
    /// If `end_offset` is past the end of the sequence,
    /// all elements from `start_offset` to the end of the sequence will be returned.
    /// If `start_offset` is past the end of the sequence or `end_offset` is less
    /// than `start_offset`, a zero-element sequence will be returned.
    ///
    /// Negative `start_offset` and `end_offset` values are allowed with arrays;
    /// in that case, the returned range counts back from the array’s end.
    /// That is, the range `(-2)` returns the last two elements, and the range of `(2,-1)`
    /// returns the second element through the next-to-last element of the range.
    /// An error will be raised on a negative `start_offset` or `end_offset` with non-arrays.
    /// (An `end_offset` of −1 is allowed with a stream if right_bound is closed;
    /// this behaves as if no `end_offset` was specified.)
    ///
    /// If `slice` is used with a [binary](crate::r::binary) object,
    /// the indexes refer to byte positions within the object.
    /// That is, the range `(10,20)` will refer to the 10th byte through the 19th byte.
    ///
    /// With a string, `slice` behaves similarly, with the indexes referring to Unicode codepoints.
    /// String indexes start at `0`.
    /// (Note that [combining codepoints](https://en.wikipedia.org/wiki/Combining_character)
    /// are counted separately.)
    ///
    /// ## Examples
    ///
    /// Return the fourth, fifth and sixth youngest players.
    /// (The youngest player is at index 0, so those are elements 3-5.)
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .order_by(r.index("age"))
    ///         .slice(args!(3, 6))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return all but the top three playerss who have a red flag.
    ///
    /// ```
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .filter(json!({"flag": "red"}))
    ///         .slice(3)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the holders of tickets `X` through `Y`,
    /// assuming tickets are numbered sequentially.
    /// We want to include ticket `Y`.
    ///
    /// ```
    /// use neor::arguments::{SliceOption, Status};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let x = 3;
    ///     let y = 6;
    ///     let slice_options = SliceOption::default().right_bound(Status::Closed);
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .order_by(r.index("ticket"))
    ///         .slice(args!(x, y, slice_options))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the elements of an array from the second through
    /// two from the end (that is, not including the last two).
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<u8> = r.expr([0, 1, 2, 3, 4, 5])
    ///         .slice(args!(2, -2))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response, vec![2, 3]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the thirds through fifth characters of a string.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: String = r.expr("I love africa.")
    ///         .slice(args!(-7, -1))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response, "africa");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [order_by](Self::order_by)
    /// - [skip](Self::skip)
    /// - [limit](Self::limit)
    /// - [nth](Self::nth)
    pub fn slice(&self, args: impl slice::SliceArg) -> Self {
        slice::new(args).with_parent(self)
    }

    /// Get the **nth** element of a sequence, counting from zero.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.nth(index) → object
    /// ```
    ///
    /// Where:
    /// - index: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If the argument is negative, count from the last element.
    ///
    /// ## Examples
    ///
    /// Select the second element in the array.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr([1, 2, 3])
    ///         .nth(1)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 2);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Select the bronze medalist from the competitors.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .order_by(r.desc("score"))
    ///         .nth(3)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Select the last place competitors.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .order_by(r.desc("score"))
    ///         .nth(-1)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [order_by](Self::order_by)
    /// - [skip](Self::skip)
    /// - [limit](Self::limit)
    /// - [bracket](Self::bracket)
    /// - [slice](Self::slice)
    pub fn nth(&self, index: impl Into<CommandArg>) -> Self {
        nth::new(index).with_parent(self)
    }

    /// Get the indexes of an element in a sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.offsets_of(datum) → array
    /// sequence.offsets_of(func!(...)) → array
    /// ```
    ///
    /// Where:
    /// - datum: `impl Serialize` | [Command](crate::Command) | [Func](crate::Func)
    ///
    /// # Description
    ///
    /// If the argument is a predicate, get
    /// the indexes of all elements matching it.
    ///
    /// ## Examples
    ///
    /// Find the position of the letter ‘c’.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Vec<usize> = r.expr(['a','b','c'])
    ///         .offsets_of('c')
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert_eq!(response.first(), Some(&2));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Find the popularity ranking of invisible heroes.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .union(r.table("dc"))
    ///         .order_by("popularity")
    ///         .offsets_of(func!(|hero| hero.g("superpowers").contains("invisibility")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn offsets_of(&self, args: impl Into<CommandArg>) -> Self {
        offsets_of::new(args).with_parent(self)
    }

    /// Test if a sequence is empty.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.is_empty() → bool
    /// ```
    ///
    /// ## Examples
    ///
    /// Are there any documents in the simbad table?
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("simbad")
    ///         .is_empty()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [offsets_of](Self::offsets_of)
    pub fn is_empty(&self) -> Self {
        is_empty::new().with_parent(self)
    }

    /// Merge two or more sequences.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// stream.union(sequence) → stream
    /// stream.union(sequences) → stream
    /// stream.union(args!(sequence, options)) → stream
    /// stream.union(args!(sequences, options)) → stream
    /// ```
    ///
    /// Where:
    /// - sequence: [Command](crate::Command)
    /// - sequences: `impl IntoIterator<Command>`
    /// - options: [UnionOption](crate::arguments::UnionOption)
    ///
    /// ## Examples
    ///
    /// Construct a stream of all characters.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("simbad")
    ///         .union(r.table("kirikou"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn union(&self, args: impl union::UnionArg) -> Self {
        union::new(args).with_parent(self)
    }

    /// Select a given number of elements from
    /// a sequence with uniform random distribution.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.sample(number) → selection
    /// stream.sample(number) → array
    /// array.sample(number) → array
    /// ```
    ///
    /// Where:
    /// - number: `u8, u16, ..., usize`
    ///
    /// # Description
    ///
    /// Select a given number of elements from a
    /// sequence with uniform random distribution.
    /// Selection is done without replacement.
    ///
    /// If the sequence has less than the requested
    /// number of elements (i.e., calling `sample(10)`
    /// on a sequence with only five elements), `sample`
    /// will return the entire sequence in a random order.
    ///
    /// ## Examples
    ///
    /// Select 3 random heroes.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("simbad")
    ///         .sample(3)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Select and stratify 3 random heroes by belovedness.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("simbad")
    ///         .group("belovedness")
    ///         .sample(3)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn sample(&self, number: impl Into<CommandArg>) -> Self {
        sample::new(number).with_parent(self)
    }

    /// Takes a stream and partitions it into multiple
    /// groups based on the fields or functions provided.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.group(field) → grouped_stream
    /// sequence.group(func) → grouped_stream
    /// sequence.group(args!(field, options)) → grouped_stream
    /// sequence.group(args!(func, options)) → grouped_stream
    /// r.group(sequence, field) → grouped_stream
    /// r.group(sequence, func) → grouped_stream
    /// r.group(sequence, args!(field, options)) → grouped_stream
    /// r.group(sequence, args!(func, options)) → grouped_stream
    /// ```
    ///
    /// Where:
    /// - field: `&str | [&str; N]`
    /// - func: [Func](crate::Func) | [Func; N]
    /// - grouped_stream: [GroupedStream](crate::types::GroupedStream)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// With the `multi` flag single documents can be assigned to multiple groups,
    /// similar to the behavior of
    /// [multi-indexes](https://rethinkdb.com/docs/secondary-indexes/javascript).
    /// When `multi` is `true` and the grouping value is an array, documents
    /// will be placed in each group that corresponds to the elements of the array.
    /// If the array is empty the row will be ignored.
    ///
    /// Suppose that the table games has the following data:
    ///
    /// ```text
    /// [
    ///     {id: 2, player: "Moussa", points: 15, class: "ranked"},
    ///     {id: 5, player: "Fatou", points: 7, class: "free"},
    ///     {id: 11, player: "Moussa", points: 10, class: "free"},
    ///     {id: 12, player: "Fatou", points: 2, class: "free"}
    /// ]
    /// ```
    ///
    /// ## Examples
    ///
    /// Group games by player.
    ///
    /// ```
    /// use neor::types::{GroupedItem, GroupedStream};
    /// use neor::{args, r, Converter, Result};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Serialize, Deserialize, PartialEq, Eq)]
    /// struct Player {
    ///     id: u8,
    ///     player: String,
    ///     points: u8,
    ///     class: String,
    /// }
    ///
    /// impl Player {
    ///     fn new(id: u8, player: &str, points: u8, class: &str) -> Self {
    ///         Self {
    ///             id,
    ///             points,
    ///             player: player.to_owned(),
    ///             class: class.to_owned(),
    ///         }
    ///     }
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let expected_data = vec![
    ///         GroupedItem {
    ///             group: String::from("Fatou"),
    ///             values: vec![
    ///                 Player::new(5, "Fatou", 7, "free"),
    ///                 Player::new(12, "Fatou", 2, "free"),
    ///             ]
    ///         },
    ///         GroupedItem {
    ///             group: String::from("Moussa"),
    ///             values: vec![
    ///                 Player::new(2, "Moussa", 15, "ranked"),
    ///                 Player::new(11, "Moussa", 10, "free"),
    ///             ]
    ///         },
    ///     ];
    ///     let conn = r.connection().connect().await?;
    ///     let response: GroupedStream<String, Player> = r.table("games")
    ///         .group("player")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.collect() == expected_data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [ungroup](Self::ungroup)
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    /// - [count](Self::count)
    /// - [sum](Self::sum)
    /// - [avg](Self::avg)
    /// - [min](Self::min)
    /// - [max](Self::max)
    pub fn group(&self, args: impl group::GroupArg) -> Self {
        group::new(args).with_parent(self)
    }

    /// Takes a grouped stream or grouped data and turns it
    /// into an array of objects representing the groups.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// grouped_stream.ungroup() → array
    /// grouped_data.ungroup() → array
    /// ```
    ///
    /// Where:
    /// - grouped_stream, grouped_data: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// Any commands chained after `ungroup` will operate on this array,
    /// rather than operating on each group individually. This is useful
    /// if you want to e.g. order the groups by the value of their reduction.
    ///
    /// The format of the array returned by `ungroup` is the same as
    /// the default native format of grouped data in the javascript driver and data explorer.
    ///
    /// Suppose that the table games has the following data:
    ///
    /// ```text
    /// [
    ///     {id: 2, player: "Moussa", points: 15, type: "ranked"},
    ///     {id: 5, player: "Fatou", points: 7, type: "free"},
    ///     {id: 11, player: "Ibrahim", points: 10, type: "free"},
    ///     {id: 12, player: "Abess", points: 2, type: "free"}
    /// ]
    /// ```
    ///
    /// ## Examples
    ///
    /// What is the maximum number of points scored by each player, with the highest scorers first?
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let expected_data = json!([
    ///         {
    ///             "group": "Bob",
    ///             "reduction": 15
    ///         },
    ///         {
    ///             "group": "Alice",
    ///             "reduction": 7
    ///         }
    ///     ]);
    ///     let response = r.table("games")
    ///         .group("player")
    ///         .max("points")
    ///         .g("points")
    ///         .ungroup()
    ///         .order_by(r.desc("reduction"))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap();
    ///
    ///     assert!(response == expected_data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// A shorter way to execute this query is to use [count](Self::count).
    ///
    /// ## Examples
    ///
    /// Suppose that each `post` has a field `comments` that is an array of comments.
    /// Return the maximum number comments per post.
    ///
    /// ```
    /// use neor::{args, func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .map(func!(|post| post.g("comments").count(())))
    ///         .reduce(func!(|left, right| r.branch(
    ///             left.gt(&right),
    ///             args!(left, right)
    ///         )))
    ///         .default(0)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// A shorter way to execute this query is to use [max](Self::max).
    ///
    /// # Related commands
    /// - [group](Self::group)
    pub fn ungroup(&self) -> Self {
        ungroup::new().with_parent(self)
    }

    /// Produce a single value from a sequence through
    /// repeated application of a reduction function.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.reduce(func) → value
    /// ```
    ///
    /// Where:
    /// - func: [Func](crate::Func)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The reduction function can be called on:
    ///
    /// - two elements of the sequence
    /// - one element of the sequence and one result of a previous reduction
    /// - two results of previous reductions
    ///
    /// The reduction function can be called on the results of
    /// two previous reductions because the `reduce` command is
    /// distributed and parallelized across shards and CPU cores.
    /// A common mistaken when using the `reduce` command is to
    /// suppose that the reduction is executed from left to right.
    /// [Read the map-reduce in RethinkDB](https://rethinkdb.com/docs/map-reduce/)
    /// article to see an example.
    ///
    /// If the sequence is empty, the server will produce a
    /// `ReqlRuntimeError` that can be caught with default.
    /// If the sequence has only one element, the first element will be returned.
    ///
    /// ## Examples
    ///
    /// Return the number of documents in the table posts.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .map(func!(|| r.expr(1)))
    ///         .reduce(func!(|left, right| left + right))
    ///         .default(0)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// A shorter way to execute this query is to use [count](Self::count).
    ///
    /// ## Examples
    ///
    /// Suppose that each `post` has a field `comments` that is an array of comments.
    /// Return the maximum number comments per post.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .map(func!(|post| post.g("comments").count(())))
    ///         .reduce(func!(|left, right| r.branch(
    ///             left.gt(&right),
    ///             args!(left, right)
    ///         )))
    ///         .default(0)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// A shorter way to execute this query is to use [max](Self::max).
    ///
    /// # Related commands
    /// - [group](Self::group)
    /// - [map](Self::map)
    /// - [concat_map](Self::concat_map)
    /// - [sum](Self::sum)
    /// - [avg](Self::avg)
    /// - [min](Self::min)
    /// - [max](Self::max)
    pub fn reduce(&self, func: Func) -> Self {
        reduce::new(func).with_parent(self)
    }

    /// Apply a function to a sequence in order,
    /// maintaining state via an accumulator.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.fold(base, func) → value
    /// ```
    ///
    /// Where:
    /// - base: `impl Serialize` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `fold` command returns either a single value or a new sequence.
    ///
    /// In its first form, `fold` operates like [reduce](Self::reduce), returning a value
    /// by applying a combining function to each element in a sequence.
    /// The combining function takes two parameters: the previous reduction
    /// result (the accumulator) and the current element. However, `fold` has
    /// the following differences from `reduce`:
    /// - it is guaranteed to proceed through the sequence from first element to last.
    /// - it passes an initial base value to the function with the first element in
    /// place of the previous reduction result.
    ///
    /// ```text
    /// combining_function(accumulator | base, element) → new_accumulator
    /// ```
    ///
    /// ## Examples
    ///
    /// Concatenate words from a list.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("words")
    ///         .order_by("id")
    ///         .fold(
    ///             "",
    ///             func!(|acc, word| acc.clone()
    ///                 + r.branch(acc.eq(""), args!("", ", "))
    ///                 + word),
    ///         )
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// (This example could be implemented with `reduce`,
    /// but `fold` will preserve the order when `words` is
    /// a RethinkDB table or other stream, which is
    /// not guaranteed with `reduce`.)
    ///
    /// # Related commands
    /// - [reduce](Self::reduce)
    /// - [concat_map](Self::concat_map)
    pub fn fold(&self, base: impl Into<CommandArg>, func: Func) -> Self {
        fold::new(base, func).with_parent(self)
    }

    /// Count the number of elements in sequence or key/value pairs in an object,
    /// or returns the size of a string or binary object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// binary.count(()) → number
    /// string.count(()) → number
    /// object.count(()) → number
    /// sequence.count(()) → number
    /// sequence.count(args!(value)) → number
    /// sequence.count(func) → number
    /// r.count(query_cmd) → number
    /// r.count(query_cmd, args!(value)) → number
    /// r.count(query_cmd, func) → number
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize`
    /// - func: [Func](crate::Func)
    /// - sequence, binary, string, object, query_cmd: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// When `count` is called on a sequence with a predicate value or function,
    /// it returns the number of elements in the sequence equal to that value or
    /// where the function returns `true`. On a [binary](crate::r::binary) object, `count`
    /// returns the size of the object in bytes; on strings, `count` returns the string’s length.
    /// This is determined by counting the number of Unicode codepoints in the string,
    /// counting combining codepoints separately.
    ///
    /// ## Examples
    ///
    /// Count the number of users.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .count(())
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Count the number of 18 year old users.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .g("age")
    ///         .count(args!(18))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Count the number of users over 18.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .g("age")
    ///         .count(func!(|age| age.gt(18)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the length of a Unicode string.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: usize = r.expr("こんにちは")
    ///         .count(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 5);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the length of an array.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: usize = r.expr(['0','1','2'])
    ///         .count(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 3);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    /// - [sum](Self::sum)
    /// - [avg](Self::avg)
    /// - [min](Self::min)
    /// - [max](Self::max)
    /// - [group](Self::group)
    pub fn count(&self, args: impl count::CountArg) -> Self {
        count::new(args).with_parent(self)
    }

    /// Sum all the elements of sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.sum(()) → number
    /// sequence.sum(field) → number
    /// sequence.sum(func) → number
    /// r.sum(sequence) → number
    /// r.sum(sequence, field) → number
    /// r.sum(sequence, func) → number
    /// ```
    ///
    /// Where:
    /// - field: `&str` | `String` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If called with a field name, sums all the values of that field in
    /// the sequence, skipping elements of the sequence that lack that field.
    /// If called with a function, calls that function on every element of the
    /// sequence and sums the results, skipping elements of the sequence
    /// where that function returns `None` or non-existence error.
    ///
    /// Returns `0` when called on an empty sequence.
    ///
    /// ## Examples
    ///
    /// What's 3 + 5 + 7?
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr([3, 5, 7])
    ///         .sum(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 15);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// How many points have been scored across all games?
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("games")
    ///         .sum("points")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// How many points have been scored across all games, counting bonus points?
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("games")
    ///         .sum(func!(|game| game.g("points") + game.g("bonus_points")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    /// - [count](Self::count)
    /// - [avg](Self::avg)
    /// - [min](Self::min)
    /// - [max](Self::max)
    /// - [group](Self::group)
    pub fn sum(&self, args: impl sum::SumArg) -> Self {
        sum::new(args).with_parent(self)
    }

    /// Averages all the elements of sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.avg(()) → Option<f64>
    /// sequence.avg(field) → Option<f64>
    /// sequence.avg(func) → Option<f64>
    /// r.avg(sequence) → Option<f64>
    /// r.avg(sequence, field) → Option<f64>
    /// r.avg(sequence, func) → Option<f64>
    /// ```
    ///
    /// Where:
    /// - field: `&str` | `String` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If called with a field name, averages all the values of that field in
    /// the sequence, skipping elements of the sequence that lack that field.
    /// If called with a function, calls that function on every element of the
    /// sequence and averages the results, skipping elements of the sequence
    /// where that function returns `None` or non-existence error.
    ///
    /// Produces a non-existence error when called on an empty sequence.
    /// You can handle this case with `default`.
    ///
    /// ## Examples
    ///
    /// What's the average of 3, 5 and 7?
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: Option<f64> = r.expr([3, 5, 7])
    ///         .avg(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == Some(5.));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// What's the average number of points scored in a games?
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("games")
    ///         .avg("points")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// What's the average number of points scored in a games, counting bonus points?
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("games")
    ///         .avg(func!(|game| game.g("points") + game.g("bonus_points")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    /// - [count](Self::count)
    /// - [sum](Self::sum)
    /// - [min](Self::min)
    /// - [max](Self::max)
    /// - [group](Self::group)
    pub fn avg(&self, args: impl avg::AvgArg) -> Self {
        avg::new(args).with_parent(self)
    }

    /// Finds the minimum element of a sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.min(()) → element
    /// sequence.min(field) → element
    /// sequence.min(func) → element
    /// sequence.min(options) → element
    /// r.min(sequence) → element
    /// r.min(sequence, field) → element
    /// r.min(sequence, func) → element
    /// r.min(sequence, options) → element
    /// ```
    ///
    /// Where:
    /// - field: `&str` | `String` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [MinOption](crate::arguments::MinOption)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `min` command can be called with:
    /// - a `field name`, to return the element of the sequence
    /// with the largest value in that field;
    /// - a `function`, to apply the function to every element within the sequence
    /// and return the element which returns the largest value from the function,
    /// ignoring any elements where the function produces a non-existence error;
    /// - an `index` (the primary key or a secondary index), to return the element
    /// of the sequence with the largest value in that index;
    ///
    /// For more information on RethinkDB’s sorting order, read the section in
    /// [ReQL data types](https://rethinkdb.com/docs/data-types/#sorting-order).
    ///
    /// Calling `min` on an empty sequence will throw a non-existence error;
    /// this can be handled using the [default](Self::default) command.
    ///
    /// ## Examples
    ///
    /// Return the minimum value in the list [3, 5, 7].
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr([3, 5, 7])
    ///         .min(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 3);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the user who has scored the fewest points.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .min("points")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// The same as above, but using a secondary index on the `points` field.
    ///
    /// ```
    /// use neor::arguments::MinOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .min(MinOption::default().index("points"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the user who has scored the fewest points,
    /// adding in bonus points from a separate field using a function.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .min(func!(|user| user.g("points") + user.g("bonus_points")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the highest number of points any user has ever scored.
    /// This returns the value of that `points` field, not a document.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.table("users")
    ///         .min("points")
    ///         .g("points")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 2);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    /// - [count](Self::count)
    /// - [sum](Self::sum)
    /// - [avg](Self::avg)
    /// - [max](Self::max)
    /// - [group](Self::group)
    pub fn min(&self, args: impl min::MinArg) -> Self {
        min::new(args).with_parent(self)
    }

    /// Finds the maximum element of a sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.max(()) → element
    /// sequence.max(field) → element
    /// sequence.max(func) → element
    /// sequence.max(options) → element
    /// r.max(sequence) → element
    /// r.max(sequence, field) → element
    /// r.max(sequence, func) → element
    /// r.max(sequence, options) → element
    /// ```
    ///
    /// Where:
    /// - field: `&str` | `String` | [Command](crate::Command)
    /// - func: [Func](crate::Func)
    /// - options: [MaxOption](crate::arguments::MaxOption)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `max` command can be called with:
    /// - a `field name`, to return the element of the sequence
    /// with the largest value in that field;
    /// - a `function`, to apply the function to every element within the sequence
    /// and return the element which returns the largest value from the function,
    /// ignoring any elements where the function produces a non-existence error;
    /// - an `index` (the primary key or a secondary index), to return the element
    /// of the sequence with the largest value in that index;
    ///
    /// For more information on RethinkDB’s sorting order, read the section in
    /// [ReQL data types](https://rethinkdb.com/docs/data-types/#sorting-order).
    ///
    /// Calling `max` on an empty sequence will throw a non-existence error;
    /// this can be handled using the [default](Self::default) command.
    ///
    /// ## Examples
    ///
    /// Return the maximum value in the list [3, 5, 7].
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr([3, 5, 7])
    ///         .max(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 7);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the user who has scored the most points.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .max("points")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// The same as above, but using a secondary index on the `points` field.
    ///
    /// ```
    /// use neor::arguments::MaxOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .max(MaxOption::default().index("points"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the user who has scored the most points,
    /// adding in bonus points from a separate field using a function.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .max(func!(|user| user.g("points") + user.g("bonus_points")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the highest number of points any user has ever scored.
    /// This returns the value of that `points` field, not a document.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.table("users")
    ///         .max("points")
    ///         .g("points")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 15);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [reduce](Self::reduce)
    /// - [count](Self::count)
    /// - [sum](Self::sum)
    /// - [avg](Self::avg)
    /// - [min](Self::min)
    /// - [group](Self::group)
    pub fn max(&self, args: impl max::MaxArg) -> Self {
        max::new(args).with_parent(self)
    }

    /// Removes duplicate elements from a sequence.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.distinct(()) → array
    /// table.distinct(options) → stream
    /// r.distinct(sequence) → array
    /// r.distinct(table, options) → stream
    /// ```
    ///
    /// Where:
    /// - options: [DistinctOption](crate::arguments::DistinctOption)
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `distinct` command can be called on any sequence or table with an index.
    ///
    /// ```text
    /// While `distinct` can be called on a table without an index,
    /// the only effect will be to convert the table into a stream;
    /// the content of the stream will not be affected.
    /// ```
    ///
    /// ## Examples
    ///
    /// Which unique villains have been vanquished by Marvel heroes?
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .concat_map(func!(|hero| hero.g("villain_list")))
    ///         .distinct(())
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Topics in a table of messages have a secondary index on them,
    /// and more than one message can have the same topic.
    /// What are the unique topics in the table?
    ///
    /// ```
    /// use neor::arguments::DistinctOption;
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("messages")
    ///         .distinct(DistinctOption::default().index("topics"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The above structure is functionally identical to:
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("messages")
    ///         .g("topics")
    ///         .distinct(())
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// However, the first form (passing the index as an argument to `distinct`) is faster,
    /// and won’t run into array limit issues since it’s returning a stream.
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [concat_map](Self::concat_map)
    /// - [group](Self::group)
    pub fn distinct(&self, args: impl distinct::DistinctArg) -> Self {
        distinct::new(args).with_parent(self)
    }

    /// When called with values, returns `true`
    /// if a sequence contains all the specified values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.contains(value) → bool
    /// query.contains(args!(values)) → bool
    /// r.contains(sequence, value) → bool
    /// r.contains(sequence, args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command) | [Func](crate::Func)
    /// - values: `impl IntoIterator<Item = T>` | `impl IntoIterator<Item = Command>`
    /// - sequence: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// When called with predicate functions, returns `true`
    /// if for each predicate there exists at least one element
    /// of the stream where that predicate returns `true`.
    ///
    /// Values and predicates may be mixed freely in the argument list.
    ///
    /// ## Examples
    ///
    /// Has Iron Man ever fought Superman?
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("marvel")
    ///         .get("ironman")
    ///         .g("opponents")
    ///         .contains("superman")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Has Iron Man ever defeated Superman in battle?
    ///
    /// ```
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("marvel")
    ///         .get("ironman")
    ///         .g("battles")
    ///         .contains(func!(|battle| battle.g("winner").eq("ironman").and(
    ///             battle.g("loser").eq("superman")
    ///         )))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return all heroes who have fought both Loki and the Hulk.
    ///
    /// ```
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("marvel")
    ///         .filter(func!(|hero| hero.g("opponents").contains(["loki", "hulk"])))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Use contains with a predicate function to simulate an or.
    /// Return the Marvel superheroes who live in Detroit, Chicago or Hoboken.
    ///
    /// ```
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("marvel")
    ///         .filter(func!(|hero| r.expr(["Detroit", "Chicago", "Hoboken"]).contains(hero.g("city"))))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    /// - [concat_map](Self::concat_map)
    /// - [group](Self::group)
    pub fn contains(&self, value: impl contains::ContainsArg) -> Self {
        contains::new(value).with_parent(self)
    }

    /// Plucks out one or more attributes from either
    /// an object or a sequence of objects (projection).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.pluck(selectors) → any
    /// ```
    ///
    /// Where:
    /// - selectors: `impl Serialize` | [Command](crate::Command) |
    /// `args!(impl IntoIterator<Item = Command>)`
    ///
    /// ## Examples
    ///
    /// We just need information about IronMan’s
    /// reactor and not the rest of the document.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .get("IronMan")
    ///         .pluck(["reactorState", "reactorPower"])
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// For the hero beauty contest we only care about certain qualities.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .pluck(["beauty", "muscleTone", "charm"])
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Pluck can also be used on nested objects.
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .pluck(json!({
    ///             "abilities": {"damage": true, "mana_cost": true},
    ///             "weapons": true
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// The nested syntax can quickly become overly
    /// verbose so there’s a shorthand for it.
    ///
    /// ```
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .pluck(args!([
    ///             r.expr(json!({"abilities": ["damage", "mana_cost"]})),
    ///             r.expr("weapons")
    ///         ]))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// For more information read the
    /// [nested field documentation](https://rethinkdb.com/docs/nested-fields/python/).
    ///
    /// # Related commands
    /// - [without](Self::without)
    /// - [map](Self::map)
    pub fn pluck(&self, args: impl pluck::PluckArg) -> Self {
        pluck::new(args).with_parent(self)
    }

    /// The opposite of pluck; takes an object or a sequence of objects,
    /// and returns them with the specified paths removed.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.without(selectors) → any
    /// ```
    ///
    /// Where:
    /// - selectors: `impl Serialize` | [Command](crate::Command) |
    /// `args!(impl IntoIterator<Item = Command>)`
    ///
    /// ## Examples
    ///
    /// Since we don’t need it for this computation we’ll save bandwidth
    /// and leave out the list of IronMan’s romantic conquests.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .get("IronMan")
    ///         .without("personalVictoriesList")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Without their prized weapons, our enemies will quickly be vanquished.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("enemies")
    ///         .without("weapons")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Nested objects can be used to remove the damage
    /// subfield from the weapons and abilities fields.
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .without(json!({
    ///             "weapons": { "damage": true },
    ///             "abilities": { "damage": true }
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// The nested syntax can quickly become overly verbose so there’s a shorthand for it.
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .without(json!({
    ///             "weapons": "damage",
    ///             "abilities": "damage"
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [pluck](Self::pluck)
    /// - [map](Self::map)
    pub fn without(&self, args: impl without::WithoutArg) -> Self {
        without::new(args).with_parent(self)
    }

    /// Merge two or more objects together to construct
    /// a new object with properties from all.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.merge(params) → any
    /// ```
    ///
    /// Where:
    /// - params: `impl Serialize` | [Func](crate::Func) | [Command](crate::Command) |
    /// `impl IntoIterator<Item = Command>` | `impl IntoIterator<Item = Func>`
    ///
    /// # Description
    ///
    /// When there is a conflict between field names, preference is
    /// given to fields in the rightmost object in the argument list
    /// `merge` also accepts a subquery function that returns an object,
    /// which will be used similarly to a [map](Self::map) function.
    ///
    /// ## Examples
    ///
    /// Equip Thor for battle.
    ///
    /// ```
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .get("thor")
    ///         .merge(args!([
    ///             r.table("equipment").get("hammer"),
    ///             r.table("equipment").get("pimento_sandwich"),
    ///         ]))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Equip every hero for battle, using a subquery
    /// function to retrieve their weapons.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("marvel")
    ///         .get("thor")
    ///         .merge(func!(|hero| {
    ///             let mut weapons = HashMap::new();
    ///             weapons.insert("weapons", r.table("weapons").get(hero.g("weapon_id")));
    ///             r.hash_map(weapons)
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Use `merge` to join each blog post with its comments.
    ///
    /// Note that the sequence being merged—in this example,
    /// the comments—must be coerced from a selection to an array.
    /// Without `coerce_to` the operation will throw an error
    /// (“Expected type DATUM but found SELECTION”).
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::arguments::GetAllOption;
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .merge(func!(|post| {
    ///             let mut comments = HashMap::new();
    ///             comments.insert("comments", r.table("comments")
    ///                 .get_all(args!(
    ///                     post.g("id"),
    ///                     GetAllOption::default().index("title")
    ///                 ))
    ///                 .coerce_to("array")
    ///             );
    ///             r.hash_map(comments)
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Merge can be used recursively to modify object within objects.
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.expr(json!({
    ///             "weapons": {
    ///                 "spectacular graviton beam": {
    ///                     "dmg": 10,
    ///                     "cooldown": 20
    ///                 }
    ///             }
    ///         }))
    ///         .merge(json!({
    ///             "weapons": {
    ///                 "spectacular graviton beam": {
    ///                     "dmg": 10
    ///                 }
    ///             }
    ///         }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [pluck](Self::pluck)
    /// - [without](Self::without)
    /// - [map](Self::map)
    pub fn merge(&self, args: impl merge::MergeArg) -> Self {
        merge::new(args).with_parent(self)
    }

    /// Append a value to an array.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.append(value) → array
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Retrieve Simon's colours list with the addition of yellow
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 6] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .append("yellow")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["green", "pink", "red", "blue", "purple", "yellow"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [prepend](Self::prepend)
    /// - [merge](Self::merge)
    /// - [insert_at](Self::insert_at)
    /// - [delete_at](Self::delete_at)
    /// - [change_at](Self::change_at)
    pub fn append(&self, args: impl Into<CommandArg>) -> Self {
        append::new(args).with_parent(self)
    }

    /// Prepend a value to an array.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.prepend(value) → array
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Retrieve Simon's colours list with the addition of yellow
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 6] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .prepend("yellow")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["yellow", "green", "pink", "red", "blue", "purple"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [append](Self::append)
    /// - [merge](Self::merge)
    /// - [insert_at](Self::insert_at)
    /// - [delete_at](Self::delete_at)
    /// - [change_at](Self::change_at)
    pub fn prepend(&self, args: impl Into<CommandArg>) -> Self {
        prepend::new(args).with_parent(self)
    }

    /// Remove the elements of one array from another array
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.difference(attr) → array
    /// ```
    ///
    /// Where:
    /// - attr: `impl IntoIterator<Item = T>` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Retrieve Simon's colour list without pink and purple
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 3] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .difference(["pink", "purple", "yellow"])
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["green", "red", "blue"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [union](Self::union)
    /// - [set_insert](Self::set_insert)
    /// - [set_union](Self::set_union)
    /// - [set_intersection](Self::set_intersection)
    /// - [set_difference](Self::set_difference)
    pub fn difference(&self, args: impl Into<CommandArg>) -> Self {
        difference::new(args).with_parent(self)
    }

    /// Add a value to an array and return it as a set
    /// (an array with distinct values).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.set_insert(value) → array
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Retrieve Simon's colours list with the addition of yellow
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 6] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .set_insert("yellow")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["green", "pink", "red", "blue", "purple", "yellow"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [union](Self::union)
    /// - [difference](Self::difference)
    /// - [set_union](Self::set_union)
    /// - [set_intersection](Self::set_intersection)
    /// - [set_difference](Self::set_difference)
    pub fn set_insert(&self, value: impl Into<CommandArg>) -> Self {
        set_insert::new(value).with_parent(self)
    }

    /// Add a several values to an array and return
    /// it as a set (an array with distinct values).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.set_union(attr) → array
    /// ```
    ///
    /// Where:
    /// - attr: `impl IntoIterator<Item = T>` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Retrieve Simon's colours list with the addition of yellow
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 6] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .set_union(["purple", "pink", "yellow"])
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["green", "pink", "red", "blue", "purple", "yellow"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [union](Self::union)
    /// - [difference](Self::difference)
    /// - [set_insert](Self::set_insert)
    /// - [set_intersection](Self::set_intersection)
    /// - [set_difference](Self::set_difference)
    pub fn set_union(&self, args: impl Into<CommandArg>) -> Self {
        set_union::new(args).with_parent(self)
    }

    /// Intersect two arrays returning values that occur in
    /// both of them as a set (an array with distinct values).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.set_intersection(attr) → array
    /// ```
    ///
    /// Where:
    /// - attr: `impl IntoIterator<Item = T>` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Check which colour Simon likes from a fixed list.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 2] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .set_intersection(["purple", "pink", "yellow"])
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["purple", "pink"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [union](Self::union)
    /// - [difference](Self::difference)
    /// - [set_insert](Self::set_insert)
    /// - [set_union](Self::set_union)
    /// - [set_difference](Self::set_difference)
    pub fn set_intersection(&self, args: impl Into<CommandArg>) -> Self {
        set_intersection::new(args).with_parent(self)
    }

    /// Remove the elements of one array from another and
    /// return them as set (an array with distinct values)
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.set_difference(attr) → array
    /// ```
    ///
    /// Where:
    /// - attr: `impl IntoIterator<Item = T>` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Check which colour Simon likes,
    /// excluding a fixed list.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // ["green", "pink", "red", "blue", "purple"]
    ///     let response: [String; 3] = r.table("simbad")
    ///         .get(1)
    ///         .g("colour")
    ///         .set_difference(["purple", "pink"])
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["green", "red", "blue"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [union](Self::union)
    /// - [difference](Self::difference)
    /// - [set_insert](Self::set_insert)
    /// - [set_union](Self::set_union)
    /// - [set_intersection](Self::set_intersection)
    pub fn set_difference(&self, args: impl Into<CommandArg>) -> Self {
        set_difference::new(args).with_parent(self)
    }

    /// Get a single field from an object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.bracket(attr) → value
    /// ```
    ///
    /// Where:
    /// - attr: `impl Into<String>` | `u8, u16, ..., usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If called on a sequence, gets that field from every object
    /// in the sequence, skipping objects that lack it.
    ///
    /// ``` text
    /// Under most circumstances, you’ll want to use [get_field](Self::get_field)
    /// (or its shorthand g) or [nth](Self::nth) rather than `bracket`.
    /// The `bracket` term may be useful in situations where you are unsure of the
    /// data type returned by the term you are calling `bracket` on.
    /// ```
    ///
    /// ## Examples
    ///
    /// How old is Moussa
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.table("simbad")
    ///         .get(1)
    ///         .bracket("age")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 15);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The `bracket` command also accepts integer arguments
    /// as array offsets, like the [nth](Self::nth) command.
    ///
    /// ## Examples
    ///
    /// How old is Moussa
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr([10, 20, 30, 40, 50])
    ///         .bracket(3)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 40);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get_field](Self::get_field)
    /// - [nth](Self::nth)
    pub fn bracket(&self, attr: impl Into<CommandArg>) -> Self {
        bracket::new(attr).with_parent(self)
    }

    /// Get a single field from an object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.get_field(attr) → value
    /// ```
    ///
    /// Where:
    /// - attr: `impl Into<String>` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If called on a sequence, gets that field from every object
    /// in the sequence, skipping objects that lack it.
    ///
    /// You may use either `get_field` or its shorthand, `g`.
    ///
    /// ## Examples
    ///
    /// How old is Moussa
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.table("simbad")
    ///         .get(1)
    ///         .get_field("age")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 15);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bracket](Self::bracket)
    /// - [nth](Self::nth)
    pub fn get_field(&self, attr: impl Into<CommandArg>) -> Self {
        get_field::new(attr).with_parent(self)
    }

    /// Get a single field from an object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.g(attr) → value
    /// ```
    ///
    /// Where:
    /// - attr: `impl Into<String>` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If called on a sequence, gets that field from every object
    /// in the sequence, skipping objects that lack it.
    ///
    /// You may use either `get_field` or its shorthand, `g`.
    ///
    /// ## Examples
    ///
    /// How old is Moussa
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.table("simbad")
    ///         .get(1)
    ///         .g("age")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 15);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bracket](Self::bracket)
    /// - [nth](Self::nth)
    pub fn g(&self, attr: impl Into<CommandArg>) -> Self {
        get_field::new(attr).with_parent(self)
    }

    /// Test if an object has one or more fields.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.has_fields(selector) → response
    /// ```
    ///
    /// Where:
    /// - selector: `impl Serialize` | [Command](crate::Command)
    /// - response: array | bool
    ///
    /// # Description
    ///
    /// An object has a field if it has that key and the key has a non-null value.
    /// For instance, the object `{'a': 1,'b': 2,'c': null}` has the fields `a` and `b`.
    ///
    /// When applied to a single object, `has_fields` returns `true` if the object has
    /// the fields and `false` if it does not. When applied to a sequence, it will return
    /// a new sequence (an array or stream) containing the elements that have the specified fields.
    ///
    /// ## Examples
    ///
    /// Return the players who have won games.
    ///
    /// ```
    /// use neor::{r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .has_fields("games_won")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the players who have not won games.
    /// To do this, use `has_fields` with [not](crate::r::not),
    /// wrapped with [filter](Self::filter).
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .filter(func!(|player| !player.has_fields("games_won")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Test if a specific player has won any games.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("players")
    ///         .get(1)
    ///         .has_fields("games_won")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Nested Fields
    ///
    /// `has_fields` lets you test for nested fields in objects.
    /// If the value of a field is itself a set of key/value pairs,
    /// you can test for the presence of specific keys.
    ///
    /// ## Examples
    ///
    /// In the `players` table, the `games_won` field contains one
    /// or more fields for kinds of games won:
    ///
    /// ```text
    /// {
    ///     'games_won': {
    ///         'playoffs': 2,
    ///         'championships': 1
    ///     }
    /// }
    /// ```
    ///
    /// Return players who have the “championships” field.
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .has_fields(json!({"games_won": {"championships": true}}))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Note that `true` in the example above is testing for the existence of `championships`
    /// as a field, not testing to see if the value of the `championships` field is set to `true`.
    /// There’s a more convenient shorthand form available.
    /// (See [pluck](Self::pluck) for more details on this.)
    ///
    /// ```
    /// use neor::{r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .has_fields(json!({"games_won": "championships"}))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get_field](Self::get_field)
    /// - [with_fields](Self::with_fields)
    pub fn has_fields(&self, selector: impl Into<CommandArg>) -> Self {
        has_fields::new(selector).with_parent(self)
    }

    /// Insert a value in to an array at a given index. Returns the modified array.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// array.insert_at(offset, value) → array
    /// ```
    ///
    /// Where:
    /// - offset: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    /// - value: `impl Serialize` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Alima decide to join Simbad.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [String; 4] = r.expr(["Moussa", "Ali", "Fati"])
    ///         .insert_at(1, "Alima")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["Moussa", "Alima", "Ali", "Fati"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [splice_at](Self::splice_at)
    /// - [change_at](Self::change_at)
    /// - [delete_at](Self::delete_at)
    pub fn insert_at<O, V>(&self, offset: O, value: V) -> Self
    where
        O: Into<CommandArg>,
        V: Into<CommandArg>,
    {
        insert_at::new(offset, value).with_parent(self)
    }

    /// Insert several values into an array at the given index.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// array.splice_at(offset, values) → array
    /// ```
    ///
    /// Where:
    /// - offset: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    /// - values: `impl IntoInterator<Item = T>` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Fati and Alima decide to join Simbad.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [String; 4] = r.expr(["Moussa", "Ali"])
    ///         .splice_at(1, ["Fati", "Alima"])
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["Moussa", "Fati", "Alima", "Ali"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [insert_at](Self::insert_at)
    /// - [delete_at](Self::delete_at)
    /// - [change_at](Self::change_at)
    pub fn splice_at<O, V>(&self, offset: O, values: V) -> Self
    where
        O: Into<CommandArg>,
        V: Into<CommandArg>,
    {
        splice_at::new(offset, values).with_parent(self)
    }

    /// Remove one or more elements from an array at a given index.
    /// (Note: `delete_at` operates on arrays, not documents;
    /// to delete documents, see the [delete](Self::delete) command.)
    ///
    /// # Command syntax
    ///
    /// ```text
    /// array.delete_at(offset) → array
    /// array.delete_at(args!(offset, end_offset)) → array
    /// ```
    ///
    /// Where:
    /// - offset, end_offset: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// If only `offset` is specified, `delete_at` removes the element at that index.
    /// If both `offset` and `end_offset` are specified, `delete_at` removes the range
    /// of elements between `offset` and `end_offset`, inclusive of `offset` but not
    /// inclusive of `end_offset`.
    ///
    /// If `end_offset` is specified, it must not be less than `offset`.
    /// Both `offset` and `end_offset` must be within the array’s bounds
    /// (i.e., if the array has 10 elements, an `offset` or `end_offset`
    /// of 10 or higher is invalid).
    ///
    /// By using a negative `offset` you can delete from the end of the array.
    /// `-1` is the last element in the array, `-2` is the second-to-last element, and so on.
    /// You may specify a negative `end_offset`, although just as with a positive value,
    /// this will not be inclusive. The range `(2,-1)` specifies the third element through
    /// the next-to-last element.
    ///
    /// ## Examples
    ///
    /// Delete the second element of an array.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [char; 5] = r.expr(['a', 'b', 'c', 'd', 'e', 'f'])
    ///         .delete_at(1)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ['a', 'c', 'd', 'e', 'f']);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete the second and third elements of an array.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [char; 4] = r.expr(['a', 'b', 'c', 'd', 'e', 'f'])
    ///         .delete_at(args!(1, 3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ['a', 'd', 'e', 'f']);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete the next-to-last element of an array.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [char; 5] = r.expr(['a', 'b', 'c', 'd', 'e', 'f'])
    ///         .delete_at(-2)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ['a', 'b', 'c', 'd', 'f']);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Delete a comment on a post.
    ///
    /// Given a post document such as:
    ///
    /// ```text
    ///
    /// ```
    /// {
    ///     "id": 1,
    ///     "title": "Post title",
    ///     "author": "Ali",
    ///     "comments": [
    ///         { "author": "Agatha", "text": "Comment 1" },
    ///         { "author": "Fatima", "text": "Comment 2" }
    ///     ]
    /// }
    ///
    /// The second comment can be deleted by using `update` and `delete_at` together.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [char; 5] = r.table("posts")
    ///         .get(1)
    ///         .update(func!(|post| {
    ///             let mut comments = HashMap::new();
    ///             comments.insert("comments", post.g("comments").delete_at(1));
    ///             r.hash_map(comments)
    ///         }))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ['a', 'b', 'c', 'd', 'f']);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [insert_at](Self::insert_at)
    /// - [splice_at](Self::splice_at)
    /// - [change_at](Self::change_at)
    pub fn delete_at(&self, args: impl delete_at::DeleteAtArg) -> Self {
        delete_at::new(args).with_parent(self)
    }

    /// Change a value in an array at a given index.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// array.change_at(offset, value) → array
    /// ```
    ///
    /// Where:
    /// - offset: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    /// - value: `impl Serialize` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Replace Ali by Alima in array.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: [String; 3] = r.expr(["Moussa", "Ali", "Fati"])
    ///         .change_at(1, "Alima")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["Moussa", "Alima", "Fati"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [insert_at](Self::insert_at)
    /// - [splice_at](Self::splice_at)
    /// - [delete_at](Self::delete_at)
    pub fn change_at<O, V>(&self, offset: O, value: V) -> Self
    where
        O: Into<CommandArg>,
        V: Into<CommandArg>,
    {
        change_at::new(offset, value).with_parent(self)
    }

    /// Return an array containing all of an object’s keys.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// string.keys() → array
    /// ```
    ///
    /// # Description
    ///
    /// Note that the keys will be sorted as described in
    /// [ReQL data types](https://rethinkdb.com/docs/data-types/#sorting-order)
    /// (for strings, lexicographically).
    ///
    /// ## Examples
    ///
    /// Get all the keys from a table row.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // row: { "id": "1", "mail": "fred@example.com", "name": "fred" }
    ///     let response: [String; 3] = r.table("users")
    ///         .get(1)
    ///         .keys()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["id", "mail", "name"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [values](Self::values)
    pub fn keys(&self) -> Self {
        keys::new().with_parent(self)
    }

    /// Return an array containing all of an object’s values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// string.values() → array
    /// ```
    ///
    /// # Description
    ///
    /// `values()` guarantees the values will
    /// come out in the same order as [keys](Self::keys).
    ///
    /// ## Examples
    ///
    /// Get all of the values from a table row.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     // row: { "id": "1", "mail": "fred@example.com", "name": "fred" }
    ///     let response: [String; 3] = r.table("users")
    ///         .get(1)
    ///         .values()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == ["1", "fred@example.com", "fred"]);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [keys](Self::keys)
    pub fn values(&self) -> Self {
        values::new().with_parent(self)
    }

    /// Match a string against a regular expression.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// string.match(regexp) → response
    /// ```
    ///
    /// Where:
    /// - regexp: `impl Into<String>` | [Command](crate::Command)
    /// - response: Option<[MatchResponse](crate::types::MatchResponse)>
    ///
    /// ## Examples
    ///
    /// ```
    /// use neor::types::MatchResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: String = r.expr("name@domain.com")
    ///         .match_(".*@(.*)")
    ///         .g("groups")
    ///         .nth(0)
    ///         .g("str")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///     let response2: Option<MatchResponse> = r.expr("name[at]domain.com")
    ///         .match_(".*@(.*)")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == "domain.com");
    ///     assert!(response2 == None);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [upcase](Self::upcase)
    /// - [downcase](Self::downcase)
    /// - [split](Self::split)
    pub fn match_(&self, regexp: impl Into<CommandArg>) -> Self {
        match_::new(regexp).with_parent(self)
    }

    /// Split a string into substrings.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// string.split(()) → string
    /// string.split(separator) → string
    /// string.split(args!(separator, max_splits)) → string
    /// ```
    ///
    /// Where:
    /// - separator: `String` | `&str` | [Commmand](crate::Command)
    /// - max_splits: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// With no arguments, will split on whitespace;
    /// when called with a string as the first argument,
    /// will split using that string as a separator.
    /// A maximum number of splits can also be specified.
    ///
    /// ## Examples
    ///
    /// Split on whitespace.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let data = [
    ///         String::from("foo"),
    ///         String::from("bar"),
    ///         String::from("bax"),
    ///     ];
    ///     let response: [String; 3] = r.expr("foo  bar bax")
    ///         .split(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Split the entries in a CSV file.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let data = [
    ///         String::from("12"),
    ///         String::from("37"),
    ///         String::new(),
    ///         String::from("22"),
    ///         String::new(),
    ///     ];
    ///     let response: [String; 5] = r.expr("12,37,,22,")
    ///         .split(",")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Split a string into characters.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let data = [
    ///         String::from("t"),
    ///         String::from("o"),
    ///         String::from("t"),
    ///         String::from("o"),
    ///     ];
    ///     let response: [String; 4] = r.expr("toto")
    ///         .split("")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Split the entries in a CSV file, but only at most 3 times.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let data = [
    ///         String::from("12"),
    ///         String::from("37"),
    ///         String::new(),
    ///         String::from("22,"),
    ///     ];
    ///     let response: [String; 4] = r.expr("12,37,,22,")
    ///         .split(args!(",", 3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Split on whitespace at most once (i.e. get the first word).
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let data = [
    ///         String::from("foo"),
    ///         String::from("bar bax"),
    ///     ];
    ///     let response: [String; 2] = r.expr("foo  bar bax")
    ///         .split(args!(" ", 1))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == data);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [upcase](Self::upcase)
    /// - [downcase](Self::downcase)
    /// - [match](Self::match_)
    pub fn split(&self, args: impl split::SplitArg) -> Self {
        split::new(args).with_parent(self)
    }

    /// Uppercases a string.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// string.upcase() → string
    /// ```
    ///
    /// ## Examples
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: String = r.expr("Sentence about LaTeX.")
    ///         .upcase()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == "SENTENCE ABOUT LATEX.");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Note
    ///
    /// `upcase` and `downcase` only affect ASCII characters.
    ///
    /// # Related commands
    /// - [downcase](Self::downcase)
    /// - [match](Self::match_)
    /// - [split](Self::split)
    pub fn upcase(&self) -> Self {
        upcase::new().with_parent(self)
    }

    /// Lowercase a string.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// string.downcase() → string
    /// ```
    ///
    /// ## Examples
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: String = r.expr("Sentence about LaTeX.")
    ///         .downcase()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == "sentence about latex.");
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Note
    ///
    /// `upcase` and `downcase` only affect ASCII characters.
    ///
    /// # Related commands
    /// - [upcase](Self::upcase)
    /// - [match](Self::match_)
    /// - [split](Self::split)
    pub fn downcase(&self) -> Self {
        downcase::new().with_parent(self)
    }

    /// Compute the logical “and” of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.and(value) → bool
    /// cmd_value.and(values) → bool
    /// r.and(values) → bool
    /// ```
    ///
    /// Where:
    /// - value: `bool` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = bool>` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `and` command can be used as an infix operator after its
    /// first argument (`r.expr(true).and(false)`) or given all of
    /// its arguments as parameters (`r.and(args!([true, false]))`).
    ///
    /// Calling `or` with zero arguments will return `false`.
    ///
    /// ## Examples
    ///
    /// Return whether either true or false evaluate to true.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.expr(true)
    ///         .or(false)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == false);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return whether any of true, true or true evaluate to true.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.or(args!([true, true, true]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [ne](Self::ne)
    /// - [or](Self::or)
    pub fn and(&self, args: impl and::AndArg) -> Self {
        and::new(args).with_parent(self)
    }

    /// Compute the logical “or” of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.or(value) → bool
    /// cmd_value.or(args!(values)) → bool
    /// r.or(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `bool` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = bool>` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `or` command can be used as an infix operator after
    /// its first argument (`r.expr(true).or(false)`) or given all
    /// of its arguments as parameters (`r.or(args!([true, false]))`).
    ///
    /// Calling `or` with zero arguments will return `false`.
    ///
    /// ## Examples
    ///
    /// Return whether either true or false evaluate to true.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.expr(true)
    ///         .or(false)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return whether any of false, false or false evaluate to false.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.or(args!([false, false, false]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == false);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Note
    ///
    /// When using `or` inside a `filter` predicate to test the values of
    /// fields that may not exist on the documents being tested,
    /// you should use the `default` command with those fields so
    /// they explicitly return `false`.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("table")
    ///         .filter(func!(|post| post.g("category")
    ///             .default("foo")
    ///             .eq("article")
    ///             .or(post.g("genre").default("foo").eq("mystery"))
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [ne](Self::ne)
    /// - [and](Self::and)
    pub fn or(&self, args: impl or::OrArg) -> Self {
        or::new(args).with_parent(self)
    }

    /// Test if two or more values are equal.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.eq(value) → bool
    /// cmd_value.eq(args!(values)) → bool
    /// r.eq(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = T>`
    ///
    /// ## Examples
    ///
    /// See if a user’s `role` field is set to `administrator`.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("users")
    ///         .get(1)
    ///         .g("role")
    ///         .eq("administrator")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// See if three variables contain equal values.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.eq(args!([20, 10, 15]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == false);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [ne](Self::ne)
    /// - [and](Self::and)
    /// - [or](Self::or)
    pub fn eq(&self, args: impl eq::EqArg) -> Self {
        eq::new(args).with_parent(self)
    }

    /// Test if two or more values are not equal.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.ne(value) → bool
    /// cmd_value.ne(args!(values)) → bool
    /// r.ne(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = T>`
    ///
    /// ## Examples
    ///
    /// See if a user’s `role` field is not set to `administrator`.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("users")
    ///         .get(1)
    ///         .g("role")
    ///         .ne("administrator")
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// See if three variables do not contain equal values.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.ne(args!([20, 10, 15]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [and](Self::and)
    /// - [or](Self::or)
    pub fn ne(&self, args: impl ne::NeArg) -> Self {
        ne::new(args).with_parent(self)
    }

    /// Compare values, testing if the left-hand value is greater than the right-hand.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.gt(value) → bool
    /// cmd_value.gt(args!(values)) → bool
    /// r.gt(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = T>`
    ///
    /// ## Examples
    ///
    /// Test if a player has scored more than 10 points.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("players")
    ///         .get(1)
    ///         .g("score")
    ///         .gt(10)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Test if variables are ordered from lowest to highest,
    /// with no values being equal to one another.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.gt(args!([20, 10, 15]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [ne](Self::ne)
    /// - [ge](Self::ge)
    /// - [lt](Self::lt)
    /// - [le](Self::le)
    pub fn gt(&self, args: impl gt::GtArg) -> Self {
        gt::new(args).with_parent(self)
    }

    /// Compare values, testing if the left-hand value is greater than the right-hand.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.ge(value) → bool
    /// cmd_value.ge(args!(values)) → bool
    /// r.ge(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = T>`
    ///
    /// ## Examples
    ///
    /// Test if a player has scored more than 10 points.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("players")
    ///         .get(1)
    ///         .g("score")
    ///         .ge(10)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Test if variables are ordered from lowest to highest,
    /// with no values being equal to one another.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.ge(args!([20, 10, 15]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [ne](Self::ne)
    /// - [gt](Self::gt)
    /// - [lt](Self::lt)
    /// - [le](Self::le)
    pub fn ge(&self, args: impl ge::GeArg) -> Self {
        ge::new(args).with_parent(self)
    }

    /// Compare values, testing if the left-hand value is less than the right-hand.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.lt(value) → bool
    /// cmd_value.lt(args!(values)) → bool
    /// r.lt(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = T>`
    ///
    /// ## Examples
    ///
    /// Test if a player has scored less than 10 points.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("players")
    ///         .get(1)
    ///         .g("score")
    ///         .lt(10)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Test if variables are ordered from highest to lowest,
    /// with no values being equal to one another.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.lt(args!([20, 10, 15]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [ne](Self::ne)
    /// - [gt](Self::gt)
    /// - [ge](Self::ge)
    /// - [le](Self::le)
    pub fn lt(&self, args: impl lt::LtArg) -> Self {
        lt::new(args).with_parent(self)
    }

    /// Compare values, testing if the left-hand value is
    /// less than or equal to the right-hand.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_value.le(value) → bool
    /// cmd_value.le(args!(values)) → bool
    /// r.le(args!(values)) → bool
    /// ```
    ///
    /// Where:
    /// - value: `impl Serialize` | [Command](crate::Command)
    /// - values: `impl IntoIterator<Item = T>`
    ///
    /// ## Examples
    ///
    /// Test if a player has scored 10 points or less.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.table("players")
    ///         .get(1)
    ///         .g("score")
    ///         .le(10)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Test if variables are ordered from highest to lowest.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: bool = r.le(args!([20, 10, 15]))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == true);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [eq](Self::eq)
    /// - [ne](Self::ne)
    /// - [gt](Self::gt)
    /// - [ge](Self::ge)
    /// - [lt](Self::lt)
    pub fn le(&self, args: impl le::LeArg) -> Self {
        le::new(args).with_parent(self)
    }

    /// Rounds the given value to the nearest whole integer.
    ///
    /// # Command syntax
    /// ```text
    /// r.round(param_number) → number
    /// number.round() → number
    /// ```
    ///
    /// Where:
    /// - param_number: `f32` | `f64` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// For example, values of 1.0 up to but not including 1.5
    /// will return 1.0, similar to [floor](Self::floor); values
    /// of 1.5 up to 2.0 will return 2.0, similar to [ceil](Self::ceil).
    ///
    /// ## Examples
    ///
    /// Round 12.345 to the nearest integer.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.round(12.345)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 12);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The `round` command can also be chained after an expression.
    ///
    /// ## Examples
    ///
    /// Round -12.345 to the nearest integer.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(-12.345)
    ///         .round()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == -12);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [ceil](Self::ceil)
    /// - [round](Self::round)
    pub fn round(&self) -> Self {
        round::new(()).with_parent(self)
    }

    /// Rounds the given value up, returning the smallest integer value
    /// greater than or equal to the given value (the value’s ceiling).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// r.ceil(param_number) → number
    /// number.ceil() → number
    /// ```
    ///
    /// Where:
    /// - param_number: `f32` | `f64` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Return the ceiling of 12.345.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.ceil(12.345)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 13);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The `ceil` command can also be chained after an expression.
    ///
    /// ## Examples
    ///
    /// Return the ceiling of -12.345.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(-12.345)
    ///         .ceil()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == -13);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [floor](Self::floor)
    /// - [round](Self::round)
    pub fn ceil(&self) -> Self {
        ceil::new(()).with_parent(self)
    }

    /// Rounds the given value down, returning the largest integer
    /// value less than or equal to the given value (the value’s floor).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// r.floor(param_number) → number
    /// number.floor() → number
    /// ```
    ///
    /// Where:
    /// - param_number: `f32` | `f64` | [Command](crate::Command)
    ///
    /// ## Examples
    ///
    /// Return the floor of 12.345.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.floor(12.345)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 13);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// The `floor` command can also be chained after an expression.
    ///
    /// ## Examples
    ///
    /// Return the floor of -12.345.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(-12.345)
    ///         .floor()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == -13);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [ceil](Self::ceil)
    /// - [round](Self::round)
    pub fn floor(&self) -> Self {
        floor::new(()).with_parent(self)
    }

    /// Compute the arithmetic "and" of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_number & cmd_number
    /// number.bit_and(param_number) → number
    /// r.bit_and(param_number, param_number) → number
    /// ```
    ///
    /// Where:
    /// - param_number: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    /// - cmd_number: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// A bitwise AND is a binary operation that takes two equal-length binary
    /// representations and performs the logical AND operation on each pair of
    /// the corresponding bits, which is equivalent to multiplying them.
    /// Thus, if both bits in the compared position are 1,
    /// the bit in the resulting binary representation is 1 (1 × 1 = 1);
    /// otherwise, the result is 0 (1 × 0 = 0 and 0 × 0 = 0).
    ///
    /// ## Examples
    ///
    /// Compute the arithmetic "and" of 5 and 3
    ///
    /// ```
    /// use std::ops::BitAnd;
    ///
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(5)
    ///         .bit_and(3)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response2: i32 = r.bit_and(r.expr(5), 3)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response3: i32 = (r.expr(5) & r.expr(3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response4: i32 = r.expr(5)
    ///         .bitand(r.expr(3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(
    ///         response == 1 &&
    ///         response == response2 &&
    ///         response == response3 &&
    ///         response == response4
    ///     );
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bit_or](Self::bit_or)
    /// - [bit_not](Self::bit_not)
    /// - [bit_xor](Self::bit_xor)
    /// - [bit_sal](Self::bit_sal)
    /// - [bit_sar](Self::bit_sar)
    pub fn bit_and(&self, number: impl Into<CommandArg>) -> Self {
        self.clone().bitand(number)
    }

    /// Compute the arithmetic "or" of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_number | cmd_number
    /// number.bit_or(param_number) → number
    /// r.bit_or(param_number, param_number) → number
    /// ```
    ///
    /// Where:
    /// - param_number: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    /// - cmd_number: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// A bitwise OR is a binary operation that takes two bit patterns
    /// of equal length and performs the logical inclusive OR operation
    /// on each pair of corresponding bits. The result in each position
    /// is 0 if both bits are 0, while otherwise the result is 1.
    ///
    /// ## Examples
    ///
    /// Compute the arithmetic "or" of 6 and 4
    ///
    /// ```
    /// use std::ops::BitOr;
    ///
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(5)
    ///         .bit_or(3)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response2: i32 = r.bit_or(r.expr(5), 3)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response3: i32 = (r.expr(5) ^ r.expr(3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response4: i32 = r.expr(5)
    ///         .bitor(r.expr(3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(
    ///         response == 7 &&
    ///         response == response2 &&
    ///         response == response3 &&
    ///         response == response4
    ///     );
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bit_and](Self::bit_and)
    /// - [bit_not](Self::bit_not)
    /// - [bit_xor](Self::bit_xor)
    /// - [bit_sal](Self::bit_sal)
    /// - [bit_sar](Self::bit_sar)
    pub fn bit_or(&self, number: impl Into<CommandArg>) -> Self {
        self.clone().bitor(number)
    }

    /// Compute the arithmetic "and" of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// cmd_number ^ cmd_number
    /// number.bit_xor(param_number) → number
    /// r.bit_xor(param_number, param_number) → number
    /// ```
    ///
    /// Where:
    /// - param_number: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    /// - cmd_number: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// A bitwise XOR is a binary operation that takes two bit patterns
    /// of equal length and performs the logical exclusive OR operation
    /// on each pair of corresponding bits. The result in each position
    /// is 1 if only the first bit is 1 or only the second bit is 1,
    /// but will be 0 if both are 0 or both are 1.
    /// In this we perform the comparison of two bits, being 1 if the
    /// two bits are different, and 0 if they are the same.
    ///
    /// ## Examples
    ///
    /// Compute the arithmetic "and" of 6 and 4
    ///
    /// ```
    /// use std::ops::BitXor;
    ///
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(6)
    ///         .bit_xor(4)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response2: i32 = r.bit_xor(r.expr(6), 4)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response3: i32 = (r.expr(6) ^ r.expr(4))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response4: i32 = r.expr(6)
    ///         .bitxor(r.expr(4))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(
    ///         response == 2 &&
    ///         response == response2 &&
    ///         response == response3 &&
    ///         response == response4
    ///     );
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bit_and](Self::bit_and)
    /// - [bit_not](Self::bit_not)
    /// - [bit_or](Self::bit_or)
    /// - [bit_sal](Self::bit_sal)
    /// - [bit_sar](Self::bit_sar)
    pub fn bit_xor(&self, number: impl Into<CommandArg>) -> Self {
        self.clone().bitxor(number)
    }

    /// Compute the arithmetic inverse (not) of an expression.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// number.bit_not() → number
    /// r.bit_not(cmd_number) → number
    /// ```
    ///
    /// Where:
    /// - cmd_number: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// A bitwise NOT, or complement, is a unary operation that performs logical
    /// negation on each bit, forming the ones’ complement of the given binary value.
    /// Bits that are 0 become 1, and those that are 1 become 0.
    ///
    /// ## Examples
    ///
    /// Negate the arithmetice expression
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: i32 = r.expr(7)
    ///         .bit_not()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response2: i32 = r.bit_not(r.expr(7))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == -8 && response == response2);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bit_and](Self::bit_and)
    /// - [bit_or](Self::bit_or)
    /// - [bit_sal](Self::bit_sal)
    /// - [bit_sar](Self::bit_sar)
    /// - [bit_xor](Self::bit_xor)
    pub fn bit_not(&self) -> Self {
        bit_not::new().with_parent(self)
    }

    /// Compute the left arithmetic shift (left logical shift) of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// number.bit_sal(param_number) → number
    /// r.bit_sal(param_number, param_number) → number
    /// ```
    ///
    /// Where:
    /// - param_number: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// In an arithmetic shift (also referred to as signed shift),
    /// like a logical shift, the bits that slide off the end disappear
    /// (except for the last, which goes into the carry flag).
    /// But in an arithmetic shift, the spaces are filled in such a way
    /// to preserve the sign of the number being slid. For this reason,
    /// arithmetic shifts are better suited for signed numbers in two’s
    /// complement format.
    ///
    /// ## Note
    ///
    /// SHL and SAL are the same, and differentiation only happens because
    /// SAR and SHR (right shifting) has differences in their implementation.
    ///
    /// ## Examples
    ///
    /// Compute the left arithmetic shift of 5 and 4
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr(5)
    ///         .bit_sar(4)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response2: u8 = r.bit_sar(r.expr(5), r.expr(4))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 80 && response == response2);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bit_and](Self::bit_and)
    /// - [bit_not](Self::bit_not)
    /// - [bit_or](Self::bit_or)
    /// - [bit_sar](Self::bit_sar)
    /// - [bit_xor](Self::bit_xor)
    pub fn bit_sal(&self, number: impl Into<CommandArg>) -> Self {
        bit_sal::new(number).with_parent(self)
    }

    /// Compute the right arithmetic shift of one or more values.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// number.bit_sar(param_number) → number
    /// r.bit_sar(param_number, param_number) → number
    /// ```
    ///
    /// Where:
    /// - param_number: `i8, u8, ..., isize, usize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// In an arithmetic shift (also referred to as signed shift),
    /// like a logical shift, the bits that slide off the end disappear
    /// (except for the last, which goes into the carry flag).
    /// But in an arithmetic shift, the spaces are filled in such
    /// a way to preserve the sign of the number being slid.
    /// For this reason, arithmetic shifts are better suited for
    /// signed numbers in two’s complement format.
    ///
    /// ## Examples
    ///
    /// Compute the right arithmetic shift of 32 and 3
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response: u8 = r.expr(32)
    ///         .bit_sar(3)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     let response2: u8 = r.bit_sar(r.expr(32), r.expr(3))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 4 && response == response2);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [bit_and](Self::bit_and)
    /// - [bit_not](Self::bit_not)
    /// - [bit_or](Self::bit_or)
    /// - [bit_sal](Self::bit_sal)
    /// - [bit_xor](Self::bit_xor)
    pub fn bit_sar(&self, number: impl Into<CommandArg>) -> Self {
        bit_sar::new(number).with_parent(self)
    }

    /// Return a new time object with a different timezone.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.in_timezone(timezone) → time
    /// ```
    ///
    /// Where:
    /// - timezone: [UtcOffset](UtcOffset)
    /// - time: [Time](crate::types::Time)
    ///
    /// # Description
    ///
    /// While the time stays the same, the results returned by methods such
    /// as hours() will change since they take the timezone into account.
    /// The timezone argument has to be of the ISO 8601 format.
    ///
    /// ## Examples
    ///
    /// Hour of the day in San Francisco (UTC/GMT -8, without daylight saving time).
    ///
    /// ```
    /// use neor::types::Time;
    /// use neor::{r, Converter, Result};
    /// use time::macros::offset;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let date_time = r.now().in_timezone(offset!(-08:00));
    ///     let time1: Time = date_time.value();
    ///     let time2: Time = date_time.cmd()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(time1.is_valid());
    ///     assert!(time2.is_valid());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [timezone](Self::timezone)
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn in_timezone(&self, timezone: UtcOffset) -> Self {
        in_timezone::new(timezone).with_parent(self)
    }

    /// Return the timezone of the time object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.timezone() → String
    /// ```
    ///
    /// ## Examples
    ///
    /// Return all the users in the “-07:00” timezone.
    ///
    /// ```
    /// use neor::{func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("subscription_date").timezone().lt("-07:00")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn timezone(&self) -> Self {
        timezone::new().with_parent(self)
    }

    /// Return whether a time is between two other times.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.during(args!(start_time, end_time)) -> bool
    /// time.during(args!(start_time, end_time, options)) -> bool
    /// ```
    ///
    /// Where:
    /// - start_time, end_time: [DateTime](crate::types::DateTime), [Command](crate::Command)
    /// - options: [DuringOption](crate::arguments::DuringOption)
    ///
    /// ## Examples
    ///
    /// Retrieve all the posts that were posted between December 1st,
    /// 2013 (inclusive) and December 10th, 2013 (exclusive).
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    /// use time::macros::{date, offset};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let start_date = r.time(args!(date!(2013 - 12 - 01), offset!(UTC)));
    ///     let end_date = r.time(args!(date!(2013 - 12 - 10), offset!(UTC)));
    ///     let response = r.table("posts")
    ///         .filter(func!(|post| post.g("date").during(args!(start_date, end_date))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve all the posts that were posted between December 1st,
    /// 2013 (exclusive) and December 10th, 2013 (inclusive).
    ///
    /// ```
    /// use neor::arguments::{DuringOption, Status};
    /// use neor::{args, func, r, Result};
    /// use time::macros::{date, offset};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let start_date = r.time(args!(date!(2013 - 12 - 01), offset!(UTC)));
    ///     let end_date = r.time(args!(date!(2013 - 12 - 10), offset!(UTC)));
    ///     let during_options = DuringOption::default()
    ///         .left_bound(Status::Open)
    ///         .right_bound(Status::Closed);
    ///     let response = r.table("posts")
    ///         .filter(func!(|post| post.g("date").during(args!(
    ///             start_date,
    ///             end_date,
    ///             during_options
    ///         ))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn during(&self, args: impl during::DuringArg) -> Self {
        during::new(args).with_parent(self)
    }

    /// Return a new time struct only based on the day,
    /// month and year (ie. the same day at 00:00).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.date() → time
    /// ```
    ///
    /// Where:
    /// - time: [Time](crate::types::Time)
    ///
    /// ## Examples
    ///
    /// Retrieve all the users whose birthday is today.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").date().eq(r.now().cmd().date())))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Note that the [now](crate::r::now) command always returns UTC time, so the
    /// comparison may fail if `user.g("birthdate")` isn’t also in UTC.
    /// You can use the [in_timezone](Self::in_timezone) command to adjust for this:
    ///
    /// ```
    /// use time::macros::offset;
    ///
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").date().eq(
    ///             r.now().cmd().in_timezone(offset!(-08:00)).date()
    ///         )))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn date(&self) -> Self {
        date::new().with_parent(self)
    }

    /// Return the number of seconds elapsed since the
    /// beginning of the day stored in the time object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.time_of_day() → f64
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve posts that were submitted before noon.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .filter(func!(|post| post.g("date").time_of_day().le(12*60*60)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn time_of_day(&self) -> Self {
        time_of_day::new().with_parent(self)
    }

    /// Return the year of a time object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.year() → i32
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve all the users born in 1986.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").year().eq(r.expr(1986))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn year(&self) -> Self {
        year::new().with_parent(self)
    }

    /// Return the month of a time object as a number between 1 and 12.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.month() → u8
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve all the users who were born in November.
    ///
    /// ```
    /// use neor::{r, func, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").month().eq(r.expr(11))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn month(&self) -> Self {
        month::new().with_parent(self)
    }

    /// Return the day of a time object as a number between 1 and 31.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.day() → u8
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the users born on the 24th of any month.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").day().eq(r.expr(24))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn day(&self) -> Self {
        day::new().with_parent(self)
    }

    /// Return the day of week of a time object as a number
    /// between 1 and 7 (following ISO 8601 standard).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.day_of_week() → u8
    /// ```
    ///
    /// ## Examples
    ///
    /// Return today’s day of week.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let day_of_week = r.now().day_of_week();
    ///     let day_of_week1 = day_of_week.value();
    ///     let day_of_week2: u8 = day_of_week.cmd()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(day_of_week1 == day_of_week2);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve all the users who were born on a Tuesday.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").day_of_week().eq(r.expr(2))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn day_of_week(&self) -> Self {
        day_of_week::new().with_parent(self)
    }

    /// Return the day of the year of a time object as a number
    /// between 1 and 366 (following ISO 8601 standard).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.day_of_year() → u16
    /// ```
    ///
    /// ## Examples
    ///
    /// Retrieve all the users who were born the first day of a year.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("birthdate").day_of_year().eq(r.expr(1))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn day_of_year(&self) -> Self {
        day_of_year::new().with_parent(self)
    }

    /// Return the hour in a time object as a number between 0 and 23.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.hours() → u8
    /// ```
    ///
    /// ## Examples
    ///
    /// Return all the posts submitted after midnight and before 4am.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .filter(func!(|post| post.g("date").hours().lt(4)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn hours(&self) -> Self {
        hours::new().with_parent(self)
    }

    /// Return the minute in a time object as a number between 0 and 59.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.minutes() → u8
    /// ```
    ///
    /// ## Examples
    ///
    /// Return all the posts submitted during the first 10 minutes of every hour.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .filter(func!(|post| post.g("date").minutes().lt(10)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn minutes(&self) -> Self {
        minutes::new().with_parent(self)
    }

    /// Return the seconds in a time object as a number between 0 and 59.999.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.seconds() → f64
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the post submitted during the first 30 seconds of every minute.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("posts")
    ///         .filter(func!(|post| post.g("date").seconds().lt(30)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [in_timezone](Self::in_timezone)
    pub fn seconds(&self) -> Self {
        seconds::new().with_parent(self)
    }

    /// Convert a time object to a string in ISO 8601 format.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.to_iso8601() → String
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the current ISO 8601 time.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let iso8601 = r.now().to_iso8601();
    ///     let iso8601_1 = iso8601.value();
    ///     let iso8601_2: String = iso8601.cmd()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(!iso8601_1.is_empty());
    ///     assert!(!iso8601_2.is_empty());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    pub fn to_iso8601(&self) -> Self {
        to_iso8601::new().with_parent(self)
    }

    /// Convert a time object to its epoch time.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// time.to_epoch_time() → f64
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the current time in seconds since
    /// the Unix Epoch with millisecond-precision.
    ///
    /// ```
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let time = r.now().to_epoch_time();
    ///     let time1: f64 = time.value();
    ///     let time2: f64 = time.cmd()
    ///         .run(&conn)
    ///         .await?.unwrap()
    ///         .parse()?;
    ///
    ///     assert!(time1.is_normal());
    ///     assert!(time2.is_normal());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [now](crate::r::now)
    /// - [time](crate::r::time)
    /// - [to_iso8601](Self::to_iso8601)
    pub fn to_epoch_time(&self) -> Self {
        to_epoch_time::new().with_parent(self)
    }

    //// Call an anonymous function using return values
    /// from other ReQL commands or queries as arguments.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// any.do_(predicate) → any
    /// r.do_(arg, predicate) → any
    /// r.do_(args, predicate) → any
    /// ```
    /// Where:
    /// - predicate: [Func](crate::Func) | [Command](crate::Command) | `impl Serialize`
    /// - arg: [Command](crate::Command)
    /// - args: `impl IntoIterator<Item = T>`
    /// - T: `impl Serialize` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The last argument to `do_` (or, in some forms, the only argument) is an
    /// expression or an anonymous function which receives values from either
    /// the previous arguments or from prefixed commands chained before `do_`.
    /// The `do_` command is essentially a single-element [map](Self::map),
    /// letting you map a function over just one document.
    /// This allows you to bind a query result to a
    /// local variable within the scope of `do_`,
    /// letting you compute the result just once and reuse it in
    /// a complex expression or in a series of ReQL commands.
    ///
    /// Arguments passed to the `do_` function must be basic data types,
    /// and cannot be streams or selections.
    /// ([Read about ReQL data types](https://rethinkdb.com/docs/data-types/).)
    /// While the arguments will all be evaluated before the function is executed,
    /// they may be evaluated in any order, so their values should not be dependent on one another.
    /// The type of `do_`’s result is the type of the value returned from the function or last expression.
    ///
    /// ## Examples
    ///
    /// Compute a golfer’s net score for a game.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///         .get("86be93eb-a112-48f5-a829-15b2cb49de1d")
    ///         .do_(func!(|player| player.g("gross_score") - player.g("course_handicap")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Return the name of the best scoring player in a two-player golf match.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let (id1, id2) = (1, 2);
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.do_(
    ///         [r.table("players").get(id1), r.table("players").get(id2)],
    ///         func!(|player1, player2| r.branch(
    ///             player1.g("gross_score").lt(player2.g("gross_score")),
    ///             args!(player1, player2)
    ///         ))
    ///     )
    ///     .run(&conn)
    ///     .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Note that `branch`, the ReQL conditional command, must be used instead of `if`.
    /// See the `branch` [documentation](Self::branch) for more.
    ///
    /// ## Examples
    ///
    /// Take different actions based on the result of a ReQL insert command.
    ///
    /// ```
    /// use neor::{args, func, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let new_data = json!({
    ///         "id": 100,
    ///         "name": "Agatha",
    ///         "gross_score": 57,
    ///         "course_handicap": 4
    ///     });
    ///     let conn = r.connection().connect().await?;
    ///     let response = r.table("players")
    ///     .insert(new_data)
    ///     .do_(func!(|doc| r.branch(
    ///         doc.g("inserted").ne(0),
    ///         args!(
    ///             r.table("log").insert(json!({"time": r.now(), "result": "ok"})),
    ///             r.table("log").insert(json!({"time": r.now(), "result": "error"}))
    ///         )
    ///     )))
    ///     .run(&conn)
    ///     .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    pub fn do_(&self, expr: impl Into<CommandArg>) -> Self {
        do_::new(expr).with_arg(self.to_owned())
    }

    /// Perform a branching conditional equivalent to `if-then-else`.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// r.branch(test, args!(true_action, false_action)) → any
    /// r.branch(test, args!(true_action, tests, false_action)) → any
    /// query.branch(args!(true_action, false_action)) -> any
    /// query.branch(args!(true_action, tests, false_action)) → any
    /// ```
    ///
    /// Where:
    /// - action, true_action, false_action: `impl Serialize` | [Command](crate::Command)
    /// - test: `bool` | [Command](crate::Command)
    /// - tests: `impl IntoIterator<Item = (test, action)>` | [Command](crate::Command)
    ///
    /// # Description
    ///
    /// The `branch` command takes 2n+1 arguments: pairs of conditional expressions
    /// and commands to be executed if the conditionals return any value but `false`
    /// or `None` i.e., “truthy” values), with a final “else” command to be evaluated
    /// if all of the conditionals are `false` or `None`.
    ///
    /// You may call `branch` infix style on the first test.
    /// (See the second example for an illustration.)
    ///
    /// ```text
    /// r.branch(test1, args!(val1, [(test2, val2)], elseval))
    /// ```
    ///
    /// is the equivalent of the Rust statement
    ///
    /// ```text
    /// if (test1) {
    ///     val1
    /// } else if (test2) {
    ///     val2
    /// } else {
    ///     elseval
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Test the value of x.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let x = 10;
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: String = r.branch(x > 5, args!("big", "small"))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.eq("big"));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// As above, infix-style.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let x = 10;
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: String = r.expr(x > 5)
    ///         .branch(args!("big", "small"))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.eq("big"));
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Categorize heroes by victory counts.
    ///
    /// ```
    /// use std::ops::Add;
    ///
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response = r.table("pricings")
    ///         .map(func!(|offer| r.branch(
    ///             offer.g("price").gt(100),
    ///             args!(
    ///                 offer.g("offer").add("premium"),
    ///                 [(
    ///                     offer.g("price").gt(10),
    ///                     offer.g("offer").add("standard")
    ///                 )],
    ///                 offer.g("offer").add("freemium")
    ///         ))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [do_](Self::do_)
    pub fn branch(&self, args: impl branch::BranchArg) -> Self {
        branch::new(args).with_parent(self)
    }

    /// Loop over a sequence, evaluating the given write query for each element.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.for_each(write_function) → response
    /// ```
    ///
    /// Where:
    /// - write_function: [Func](crate::Func)
    /// - response: [MutationResponse](crate::types::MutationResponse)
    ///
    /// ## Examples
    ///
    /// Get information about a table such as primary key, or cache size.
    ///
    /// ```
    /// use neor::types::MutationResponse;
    /// use neor::{args, func, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: MutationResponse = r.table("models")
    ///         .for_each(func!(|model| r.table("cars")
    ///             .get(model.get("car_model"))
    ///             .delete(())
    ///         ))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.deleted == 5);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [map](Self::map)
    pub fn for_each(&self, write_function: Func) -> Self {
        for_each::new(write_function).with_parent(self)
    }

    /// Provide a default value in case of non-existence errors.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// value.default(default_value) → any
    /// sequence.default(default_value) → any
    /// ```
    ///
    /// Where:
    /// - default_value: `impl Serialize` | [Command](crate::Command) | [Func](crate::Func)
    ///
    /// # Description
    ///
    /// The `default` command evaluates its first argument (the value it’s chained to).
    /// If that argument returns `None` or a non-existence error is thrown in evaluation,
    /// then `default` returns its second argument. The second argument is usually a default value,
    /// but it can be a function that returns a value.
    ///
    /// ## Examples
    ///
    /// Suppose we want to retrieve the titles and authors of the table posts.
    /// In the case where the author field is missing or null,
    /// we want to retrieve the string Anonymous.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::{func, r, Converter, Result};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Post {
    ///     title: String,
    ///     author: String,
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: Vec<Post> = r.table("posts")
    ///         .map(func!(|doc| {
    ///             let mut post = HashMap::new();
    ///             post.insert("title", doc.g("title"));
    ///             post.insert("author", doc.g("author").default("Anonymous"));
    ///             r.hash_map(post)
    ///         }))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.len() > 0);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// The `default` command can also be used to filter documents.
    /// Suppose we want to retrieve all our users who are not grown-ups or
    /// whose age is unknown (i.e., the field `age` is missing or equals `None`).
    /// We can do it with this query:
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("age").lt(18).default(true)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// One more way to write the previous query is
    /// to set the age to be `-1` when the field is missing.
    ///
    /// ```
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.g("age").default(-1).lt(18)))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Another way to do the same query is to use hasFields.
    ///
    /// ```
    /// use std::ops::Not;
    ///
    /// use neor::{func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response = r.table("users")
    ///         .filter(func!(|user| user.has_fields("age").not().or(user.g("age").lt(18))))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// Another way to do the same query is to use hasFields.
    ///
    /// ```
    /// use neor::arguments::FilterOption;
    /// use neor::{args, func, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response = r.table("users")
    ///         .filter(args!(
    ///             func!(|user| user.g("age").lt(18).default(true)),
    ///             FilterOption::default().default_(true)
    ///         ))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn default(&self, default_value: impl Into<CommandArg>) -> Self {
        default::new(default_value).with_parent(self)
    }

    /// Convert a value of one type into another.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// sequence.coerce_to('array') → array
    /// value.coerce_to('string') → string
    /// string.coerce_to('number') → number
    /// array.coerce_to('object') → object
    /// sequence.coerce_to('object') → object
    /// object.coerce_to('array') → array
    /// binary.coerce_to('string') → string
    /// string.coerce_to('binary') → binary
    ///
    /// query.coerce_to(type) → response
    /// ```
    ///
    /// Where:
    /// - type: [CoerceType](crate::arguments::CoerceType) | `impl Into<String>` |
    /// [Command](crate::Command)
    ///
    /// # Description
    ///
    /// - a sequence, selection or object can be coerced to an array
    /// - a sequence, selection or an array of key-value pairs can be coerced to an object
    /// - a string can be coerced to a number
    /// - any datum (single value) can be coerced to a string
    /// - a binary object can be coerced to a string and vice-versa
    ///
    /// ## Examples
    ///
    /// Coerce an array of pairs into an object.
    ///
    /// ```
    /// use neor::arguments::CoerceType;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response = r.expr([["name", "Malika"], ["genre", "woman"]])
    ///         .coerce_to(CoerceType::Object)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Coerce a number to a string.
    ///
    /// ```
    /// use neor::arguments::CoerceType;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: char = r.expr(1)
    ///         .coerce_to(CoerceType::String)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == '1');
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [object](crate::r::object)
    pub fn coerce_to(&self, value: impl Into<CommandArg>) -> Self {
        coerce_to::new(value).with_parent(self)
    }

    /// Gets the type of a ReQL query’s return value.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// any.type_of() → response
    /// ```
    ///
    /// Where:
    /// - response: [TypeOf](crate::types::TypeOf)
    ///
    /// # Description
    ///
    /// Read the article on [ReQL data types](https://rethinkdb.com/docs/data-types/)
    /// for a more detailed discussion.
    /// Note that some possible return values from `type_of` are internal values,
    /// such as `TypeOf::MAXVAL`, and unlikely to be returned from queries in standard practice.
    ///
    /// ## Examples
    ///
    /// Get the type of a TypeOf.
    ///
    /// ```
    /// use neor::types::TypeOf;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: TypeOf = r.expr("foo")
    ///         .type_of()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == TypeOf::String);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn type_of(&self) -> Self {
        type_of::new().with_parent(self)
    }

    /// Get information about a ReQL value.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// any.info() → response
    /// r.info(any) → response
    /// ```
    ///
    /// Where:
    /// - response: [InfoResponse](crate::types::InfoResponse)
    ///
    /// ## Examples
    ///
    /// Get information about a table such as primary key, or cache size.
    ///
    /// ```
    /// use neor::types::{InfoResponse, TypeOf};
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: InfoResponse = r.table("simbad")
    ///         .info()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.typ == TypeOf::Table);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn info(&self) -> Self {
        info::new().with_parent(self)
    }

    /// Convert a ReQL value or object to a JSON string.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// value.to_json() → String
    /// ```
    ///
    /// ## Examples
    ///
    /// Get a ReQL document as a JSON string.
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: String = r.table("simbad")
    ///         .get(1)
    ///         .to_json()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(!response.is_empty());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn to_json(&self) -> Self {
        to_json::new().with_parent(self)
    }

    /// Compute the distance between a point and another geometry object.
    /// At least one of the geometry objects specified must be a point.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// geometry.distance(geometry) → f64
    /// geometry.distance(args!(geometry, options)) → f64
    /// r.distance(geometry.cmd(), geometry) → f64
    /// r.distance(geometry.cmd(), args!(geometry, options)) → f64
    /// ```
    ///
    /// Where:
    /// - geometry: [r.point(...)](crate::r::point) |
    /// [r.line(...)](crate::r::line) |
    /// [r.polygon(...)](crate::r::polygon)
    /// - options: [DistanceOption](crate::arguments::DistanceOption)
    ///
    /// # Description
    ///
    /// If one of the objects is a polygon or a line, the point will be projected
    /// into the line or polygon assuming a perfect sphere model before the distance
    /// is computed (using the model specified with `geo_system`).
    /// As a consequence, if the polygon or line is extremely large compared
    /// to Earth’s radius and the distance is being computed with the default
    ///  WGS84 model, the results of `distance` should be considered approximate
    /// due to the deviation between the ellipsoid and spherical models.
    ///
    /// ## Examples
    ///
    /// Compute the distance between two points on the Earth in kilometers.
    ///
    /// ```
    /// use neor::arguments::{DistanceOption, Unit};
    /// use neor::{args, r, Converter, Geometry, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point1 = r.point(-122.423246, 37.779388);
    ///     let point2 = r.point(-117.220406, 32.719464);
    ///     let distance_option = DistanceOption::default().unit(Unit::Kilometer);
    ///
    ///     let response: f64 = point1.cmd()
    ///         .distance(point2)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response == 734.125249602186);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [polygon](crate::r::polygon)
    /// - [line](crate::r::line)
    pub fn distance(&self, args: impl distance::DistanceArg) -> Self {
        distance::new(args).with_parent(self)
    }

    /// Convert a ReQL geometry object to a [GeoJSON](https://geojson.org/) object.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// geometry.to_geojson() → response
    /// ```
    ///
    /// Where:
    /// - geometry: [r.point(...)](crate::r::point) |
    /// [r.line(...)](crate::r::line) |
    /// [r.polygon(...)](crate::r::polygon)
    /// command
    /// - response: [GeoJson<T>](crate::types::GeoJson)
    ///
    /// ## Examples
    ///
    /// Convert a ReQL geometry object to a GeoJSON object.
    ///
    /// ```
    /// use neor::types::{GeoJson, GeoType};
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: GeoJson<[f64; 2]> = r.table("simbad")
    ///         .get("sfo")
    ///         .g("location")
    ///         .to_geojson()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.typ == GeoType::Point);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [geojson](crate::r::geojson)
    pub fn to_geojson(&self) -> Self {
        to_geojson::new().with_parent(self)
    }

    /// Get all documents where the given geometry object intersects
    /// the geometry object of the requested geospatial index.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.get_intersecting(geometry, options) → selection<stream>
    /// ```
    ///
    /// Where:
    /// - geometry: [r.point(...)](crate::r::point) |
    /// [r.line(...)](crate::r::line) |
    /// [r.polygon(...)](crate::r::polygon)
    /// command
    /// - sequence: command
    /// - options: [GetIntersectingOption](crate::arguments::GetIntersectingOption)
    ///
    /// # Description
    ///
    /// The `index` argument is mandatory. This command returns the same
    ///  results as `|row| row.g(index).intersects(geometry)`.
    /// The total number of results is limited to the array size limit
    /// which defaults to 100,000, but can be changed with the `array_limit`
    /// option to [run](Self::run).
    ///
    /// ## Examples
    ///
    /// Which of the locations in a list of parks intersect `circle`?
    ///
    /// ```
    /// use neor::arguments::{CircleOption, Unit};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point = r.point(-117.220406, 32.719464);
    ///     let circle_opts = CircleOption::default()
    ///         .unit(Unit::InternationalMile);
    ///     let circle = r.circle(args!(point, 10., circle_opts));
    ///
    ///     let response = r.table("simbad")
    ///         .get_intersecting(circle, "area")
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get_nearest](Self::get_nearest)
    pub fn get_intersecting(
        &self,
        geometry: impl get_intersecting::GetIntersectingArg,
        index: &'static str,
    ) -> Self {
        get_intersecting::new(geometry, index).with_parent(self)
    }

    /// Return a list of documents closest to a
    /// specified point based on a geospatial index,
    /// sorted in order of increasing distance.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.get_nearest(args!(geometry, &str)) → array
    /// table.get_nearest(args!(geometry, &str, options)) → array
    /// ```
    ///
    /// Where:
    /// - geometry: [r.point(...)](crate::r::point) |
    /// [r.line(...)](crate::r::line) |
    /// [r.polygon(...)](crate::r::polygon) |
    /// command
    /// - sequence: command
    /// - options: [GetNearestOption](crate::arguments::GetNearestOption)
    ///
    /// # Description
    ///
    /// The return value will be an array of two-item objects
    /// with the keys `dist` and `doc`, set to the distance
    /// between the specified point and the document
    /// (in the units specified with `unit`, defaulting to meters)
    /// and the document itself, respectively.
    /// The array will be sorted by the values of `dist`.
    ///
    /// ## Examples
    ///
    /// Return a list of the closest 25 enemy hideouts to the secret base.
    ///
    /// ```
    /// use neor::arguments::GetNearestOption;
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let secret_base = r.point(-122.422876, 37.777128);
    ///     let opts = GetNearestOption::default().max_results(25);
    ///
    ///     let response = r.table("simbad")
    ///         .get_nearest(args!(secret_base, "location"))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [get_intersecting](Self::get_intersecting)
    pub fn get_nearest(&self, args: impl get_nearest::GetNearestArg) -> Self {
        get_nearest::new(args).with_parent(self)
    }

    /// Tests whether a geometry object is completely contained within another.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// geometry.includes(geometry) → bool
    /// sequence.includes(geometry) → sequence
    /// ```
    ///
    /// Where:
    /// - geometry: [r.point(...)](crate::r::point) |
    /// [r.line(...)](crate::r::line) |
    /// [r.polygon(...)](crate::r::polygon) |
    /// command
    /// - sequence: command
    ///
    /// # Description
    ///
    /// When applied to a sequence of geometry objects,
    /// `includes` acts as a [filter](Self::filter),
    /// returning a sequence of objects from the sequence
    /// that include the argument.
    ///
    /// ## Examples
    ///
    /// Is `point2` included within a 2000-meter circle around `point1`?
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point1 = r.point(-117.220406, 32.719464);
    ///     let point2 = r.point(-117.206201, 32.725186);
    ///
    ///     let response: bool = r.circle(args!(point1, 2000.))
    ///         .includes(point2)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Which of the locations in a list of parks include `circle`?
    ///
    /// ```
    /// use neor::arguments::{CircleOption, Unit};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point = r.point(-117.220406, 32.719464);
    ///     let circle_opts = CircleOption::default().unit(Unit::InternationalMile);
    ///     let circle = r.circle(args!(point, 10., circle_opts));
    ///
    ///     let response = r.table("parks")
    ///         .g("area")
    ///         .includes(circle)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Rewrite the previous example with `get_intersecting`.
    ///
    /// ```
    /// use neor::arguments::{CircleOption, Unit};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point = r.point(-117.220406, 32.719464);
    ///     let circle_opts = CircleOption::default()
    ///         .unit(Unit::InternationalMile);
    ///     let circle = r.circle(args!(point, 10., circle_opts));
    ///
    ///     let response = r.table("parks")
    ///         .get_intersecting(circle.clone(), "area")
    ///         .g("area")
    ///         .includes(circle)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [intersects](Self::intersects)
    pub fn includes(&self, args: impl includes::IncludesArg) -> Self {
        includes::new(args).with_parent(self)
    }

    /// Tests whether two geometry objects intersect with one another.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// geometry.intersects(geometry) → bool
    /// r.intersects(geometry_command, geometry) → bool
    /// sequence.intersects(geometry) → sequence_response
    /// r.intersects(sequence, geometry) → sequence_response
    /// ```
    ///
    /// Where:
    /// - geometry: [r.point(...)](crate::r::point) |
    /// [r.line(...)](crate::r::line) |
    /// [r.polygon(...)](crate::r::polygon) |
    /// command
    /// - sequence, geometry_command: [Command](crate::Command)
    ///
    /// # Description
    ///
    /// When applied to a sequence of geometry objects, `intersects` acts as a
    /// [filter](Self::filter), returning a sequence of objects from
    /// the sequence that intersect with the argument.
    ///
    /// ## Examples
    ///
    /// Is `point2` within a 2000-meter circle around `point1`?
    ///
    /// ```
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point1 = r.point(-117.220406, 32.719464);
    ///     let point2 = r.point(-117.206201, 32.725186);
    ///
    ///     let response: bool = r.circle(args!(point1, 2000.))
    ///         .intersects(point2)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Which of the locations in a list of parks intersect `circle`?
    ///
    /// ```
    /// use neor::arguments::{CircleOption, Unit};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let point = r.point(-117.220406, 32.719464);
    ///     let circle_opts = CircleOption::default().unit(Unit::InternationalMile);
    ///     let circle = r.circle(args!(point, 10., circle_opts));
    ///
    ///     let response = r.table("parks")
    ///         .g("area")
    ///         .intersects(circle)
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     assert!(response.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [includes](Self::includes)
    /// - [get_intersecting](Self::get_intersecting)
    pub fn intersects(&self, geometry: impl intersects::IntersectsArg) -> Self {
        intersects::new(geometry).with_parent(self)
    }

    /// Grant or deny access permissions for a user account,
    /// globally or on a per-database or per-table basis.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// r.grant(username, permission) → response
    /// table.grant(username, permission) → response
    /// db.grant(username, permission) → response
    /// ```
    ///
    /// Where:
    /// - username: `impl Serialize`
    /// - permission: [Permission](crate::arguments::Permission)
    /// - response: [GrantResponse](crate::types::GrantResponse)
    ///
    /// # Description
    ///
    /// Permissions that are not defined on a local scope will
    /// be inherited from the next largest scope.
    /// For example, a write operation on a table will first
    /// check if `write` permissions are explicitly set to `true` or `false`
    /// for that table and account combination; if they are not,
    /// the `write` permissions for the database will be used
    /// if those are explicitly set; and if neither table nor database
    /// permissions are set for that account, the global `write`
    /// permissions for that account will be used.
    ///
    /// ## Note
    ///
    /// For all accounts other than the special, system-defined `admin` account,
    /// permissions that are not explicitly set in any scope will effectively be `false`.
    /// When you create a new user account by inserting a record into the
    /// [system table](https://rethinkdb.com/docs/system-tables/#users),
    /// that account will have **no** permissions until they are explicitly granted.
    ///
    /// For a full description of permissions, read
    /// [Permissions and user accounts](https://rethinkdb.com/docs/permissions-and-accounts/).
    ///
    /// ## Examples
    ///
    /// Grant the `alima` user account read and write permissions on the `users` database.
    ///
    /// ```
    /// use neor::arguments::Permission;
    /// use neor::types::GrantResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let permission = Permission::default().read(true).write(true);
    ///
    ///     let response: GrantResponse = r.db("users")
    ///         .grant("alima", permission)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.granted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Deny write permissions from the `alima` account for the `simbad` table.
    ///
    /// ```
    /// use neor::arguments::Permission;
    /// use neor::types::GrantResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let permission = Permission::default().write(false);
    ///
    ///     let response: GrantResponse = r.db("users")
    ///         .table("simbad")
    ///         .grant("alima", permission)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.granted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Grant `alima` the ability to use HTTP connections.
    ///
    /// ```
    /// use neor::arguments::Permission;
    /// use neor::types::GrantResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let permission = Permission::default().connect(true);
    ///
    ///     let response: GrantResponse = r.grant("alima", permission)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.granted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Grant a `monitor` account read-only access to all databases.
    ///
    /// ```
    /// use neor::arguments::Permission;
    /// use neor::types::GrantResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let permission = Permission::default().read(true);
    ///
    ///     let response: GrantResponse = r.grant("monitor", permission)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.granted == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn grant(&self, username: impl Into<String>, permission: Permission) -> Self {
        grant::new(username, permission).with_parent(self)
    }

    /// Query (read and/or update) the configurations for individual tables or databases.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.config() → response
    /// db.config() → response
    /// ```
    ///
    /// Where:
    /// - table: [r.table(...)](crate::r::table) |
    /// [query.table(...)](Self::table)
    /// - db: [r.db(...)](crate::r::db)
    /// - response: [ConfigResponse](crate::types::ConfigResponse)
    ///
    /// # Description
    ///
    /// The config command is a shorthand way to access the `table_config` or `db_config`
    /// [System tables](https://rethinkdb.com/docs/system-tables/#configuration-tables).
    /// It will return the single row from the system that corresponds to the database
    /// or table configuration, as if [get](Self::get) had been called on the system
    /// table with the UUID of the database or table in question.
    ///
    /// ## Examples
    ///
    /// Rebalance a table.
    ///
    /// ```
    /// use neor::types::ConfigResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: ConfigResponse = r.table("simbad")
    ///         .config()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.name == "simbad");
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn config(&self) -> Self {
        config::new().with_parent(self)
    }

    /// Rebalances the shards of a table. When called on a database,
    /// all the tables in that database will be rebalanced.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.rebalance() → response
    /// db.rebalance() → response
    /// ```
    ///
    /// Where:
    /// - table: [r.table(...)](crate::r::table) |
    /// [query.table(...)](Self::table)
    /// - db: [r.db(...)](crate::r::db)
    /// - response: [RebalanceResponse](crate::types::RebalanceResponse)
    ///
    /// # Description
    ///
    /// The `rebalance` command operates by measuring the distribution of
    /// primary keys within a table and picking split points that will
    /// give each shard approximately the same number of documents.
    /// It won’t change the number of shards within a table,
    /// or change any other configuration aspect for the table or the database.
    ///
    /// A table will lose availability temporarily after `rebalance` is called;
    /// use the [wait](Self::wait) command to wait for the table to become available again,
    /// or [status](Self::status) to check if the table is available for writing.
    ///
    /// RethinkDB automatically rebalances tables when the number of shards are increased,
    /// and as long as your documents have evenly distributed primary keys—such as
    /// the default UUIDs—it is rarely necessary to call `rebalance` manually.
    /// Cases where `rebalance` may need to be called include:
    /// - Tables with unevenly distributed primary keys, such as incrementing integers
    /// - Changing a table’s primary key type
    /// - Increasing the number of shards on an empty table,
    /// then using non-UUID primary keys in that table
    ///
    /// The [web UI](https://rethinkdb.com/docs/administration-tools/)
    /// (and the [info](Self::info) command)
    /// can be used to tell you when a table’s shards need to be rebalanced.
    ///
    /// See the [status](Self::status) command for an explanation of
    /// the objects returned in the `old_val` and `new_val` fields.
    ///
    /// ## Examples
    ///
    /// Rebalance a table.
    ///
    /// ```
    /// use neor::types::RebalanceResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: RebalanceResponse = r.table("simbad")
    ///         .rebalance()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.rebalanced == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn rebalance(&self) -> Self {
        rebalance::new().with_parent(self)
    }

    /// Reconfigure a table’s sharding and replication.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.reconfigure(options) → response
    /// db.reconfigure(options) → response
    /// ```
    ///
    /// Where:
    /// - table: [r.table(...)](crate::r::table) |
    /// [query.table(...)](Self::table)
    /// - db: [r.db(...)](crate::r::db)
    /// - options: [ReconfigureOption](crate::arguments::ReconfigureOption)
    /// - response: [ReconfigureResponse](crate::types::ReconfigureResponse)
    ///
    /// # Description
    ///
    /// A table will lose availability temporarily after `reconfigure` is called;
    /// use the [wait](Self::wait) command to wait for the table to become available again,
    /// or [status](Self::wait) to check if the table is available for writing.
    ///
    /// ## Note
    ///
    /// Whenever you call `reconfigure`, the write durability will be set to
    /// `Durability::Hard` and the write
    /// acknowledgments will be set to `ReadMode::Majority`;
    /// these can be changed by using the `config` command on the table.
    ///
    ///
    /// If `reconfigure` is called on a database,
    /// all the tables in the database will have their configurations affected.
    /// The return value will be an array of the objects described above, one per table.
    ///
    /// Read [Sharding and replication](https://rethinkdb.com/docs/sharding-and-replication/)
    /// for a complete discussion of the subject, including advanced topics.
    ///
    /// ## Examples
    ///
    /// Reconfigure a table.
    ///
    /// ```
    /// use neor::arguments::{ReconfigureOption, Replicas};
    /// use neor::types::ReconfigureResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = ReconfigureOption::default()
    ///         .shards(1)
    ///         .replicas(Replicas::Int(1));
    ///
    ///     let response: ReconfigureResponse = r.table("simbad")
    ///         .reconfigure(opts)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.reconfigured == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Reconfigure a table, specifying replicas by server tags.
    ///
    /// ```
    /// use std::collections::HashMap;
    ///
    /// use neor::arguments::{ReconfigureOption, Replicas};
    /// use neor::types::ReconfigureResponse;
    /// use neor::{r, Converter, Result, StaticString};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let mut replicas = HashMap::new();
    ///
    ///     replicas.insert("malika".static_string(), 1);
    ///     replicas.insert("malika".static_string(), 1);
    ///
    ///     let opts = ReconfigureOption::default()
    ///         .shards(2)
    ///         .replicas(Replicas::Map {
    ///             replicas,
    ///             primary_replica_tag: "malika".static_string()
    ///         });
    ///
    ///     let response: ReconfigureResponse = r.table("simbad")
    ///         .reconfigure(opts)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.reconfigured == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Perform an emergency repair on a table.
    ///
    /// ```
    /// use neor::arguments::{EmergencyRepair, ReconfigureOption};
    /// use neor::types::ReconfigureResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = ReconfigureOption::default()
    ///         .emergency_repair(EmergencyRepair::UnsafeRollback);
    ///
    ///     let response: ReconfigureResponse = r.table("simbad")
    ///         .reconfigure(opts)
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.reconfigured == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn reconfigure(&self, opts: ReconfigureOption) -> Self {
        reconfigure::new(opts).with_parent(self)
    }

    /// Return the status of a table.
    ///
    /// The return value is an object providing information about
    /// the table’s shards, replicas and replica readiness states.
    /// For a more complete discussion of the object fields,
    /// read about the table_status table in
    /// [System tables](https://rethinkdb.com/docs/system-tables/#status-tables).
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.status() → response
    /// ```
    ///
    /// Where:
    /// - table: [r.table(...)](crate::r::table) |
    /// [query.table(...)](Self::table)
    /// - response: [StatusResponse](crate::types::StatusResponse)
    ///
    /// ## Examples
    ///
    /// Get a table’s status.
    ///
    /// ```
    /// use neor::types::StatusResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: StatusResponse = r.table("simbad")
    ///         .status()
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.name.is_some());
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn status(&self) -> Self {
        status::new().with_parent(self)
    }

    /// Wait for a table or all the tables in a database to be ready.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// table.wait(()) → response
    /// db.wait(()) → response
    /// r.wait(table) → response
    /// r.wait(database) → response
    /// table.wait(options) → response
    /// db.wait(options) → response
    /// r.wait(args!(table, options)) → response
    /// r.wait(args!(database, options)) → response
    /// ```
    ///
    /// Where:
    /// - table: [r.table(...)](crate::r::table) |
    /// [query.table(...)](Self::table)
    /// - db: [r.db(...)](crate::r::db)
    /// - options: [WaitOption](crate::arguments::WaitOption)
    /// - response: [WaitResponse](crate::types::WaitResponse)
    ///
    /// # Description
    ///
    /// A table may be temporarily unavailable after creation,
    /// rebalancing or reconfiguring.
    /// The `wait` command blocks until the given
    /// table (or database) is fully up to date.
    ///
    /// ## Examples
    ///
    /// Wait on a table to be ready.
    ///
    /// ```
    /// use neor::types::WaitResponse;
    /// use neor::{r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     let response: WaitResponse = r.table("simbad")
    ///         .wait(())
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.ready == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Wait on a table with timeout to be ready for reads.
    ///
    /// ```
    /// use neor::arguments::{WaitFor, WaitOption};
    /// use neor::types::WaitResponse;
    /// use neor::{args, r, Converter, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let table_command = r.db("anim").table("simbad");
    ///     let opts = WaitOption::default()
    ///         .wait_for(WaitFor::ReadyForReads)
    ///         .timeout(8000f64);
    ///
    ///     let response: WaitResponse =  r.wait(args!(table_command, opts))
    ///         .run(&conn)
    ///         .await?
    ///         .unwrap()
    ///         .parse()?;
    ///
    ///     assert!(response.ready == 1);
    ///     
    ///     Ok(())
    /// }
    /// ```
    pub fn wait(&self, args: impl wait::WaitArg) -> Self {
        wait::new(args).with_parent(self)
    }

    /// Run a query on a connection,
    /// returning either a single JSON result or a cursor,
    /// depending on the query.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.run(&session) → value
    /// query.run(connection) → value
    /// query.run(args!(&session, options)) → value
    /// query.run(args!(connection, options)) → value
    /// query.run(&mut session) → value
    /// query.run(args!(&mut session, options)) → value
    /// ```
    ///
    /// Where:
    /// - session: [Session](crate::connection::Session)
    /// - connection: [Connection](crate::connection::Connection)
    /// - options: [RunOption](crate::arguments::RunOption)
    /// - stream: [impl Stream<Item = Result<Value>>](futures::stream::Stream)
    ///
    /// ## Examples
    ///
    /// If you are OK with potentially out of date data
    /// from all the tables involved in this query and
    /// want potentially faster reads,
    /// pass a flag allowing out of date data in an options object.
    /// Settings for individual tables will supercede this global
    /// setting for all tables in the query.
    ///
    /// ```
    /// use neor::arguments::{ReadMode, RunOption};
    /// use neor::{args, r, Result};
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = RunOption::default().read_mode(ReadMode::Outdated);
    ///
    ///     r.table("simbad").run(args!(&conn, opts)).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// If you want to specify whether to wait for a write to be written
    /// to disk (overriding the table’s default settings),
    /// you can set `durability` to `Durability::Hard`
    /// or `Durability::Soft` in the options.
    ///
    /// ```
    /// use neor::arguments::{Durability, RunOption};
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = RunOption::default().durability(Durability::Hard);
    ///     let data = json!({
    ///         "name": "Pumba",
    ///         "live": 5
    ///     });
    ///
    ///     r.table("simbad").insert(data).run(args!(&conn, opts)).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// If you do not want a time object to be converted to a native date object,
    /// you can pass a time_format flag to prevent it
    /// (valid flags are `Format::Raw` and `Format::Native`).
    /// This query returns an object with two fields (epoch_time and $reql_type$)
    /// instead of a native date object.
    ///
    /// ```
    /// use neor::arguments::{Format, RunOption};
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = RunOption::default().time_format(Format::Raw);
    ///
    ///     r.now().cmd().run(args!(&conn, opts)).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Specify the database to use for the query.
    ///
    /// ```
    /// use neor::arguments::{Format, RunOption};
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = RunOption::default().db("jikoni");
    ///
    ///     r.table("simbad").run(args!(&conn, opts)).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Change the batching parameters for this query.
    ///
    /// ```
    /// use neor::arguments::{Format, RunOption};
    /// use neor::{args, r, Result};
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let opts = RunOption::default()
    ///         .max_batch_rows(16)
    ///         .max_batch_bytes(2048);
    ///
    ///     r.table("simbad").run(args!(&conn, opts)).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [connection](crate::r::connection)
    pub async fn run(&self, args: impl run::RunArg) -> Result<Option<Value>> {
        self.build_query(args).try_next().await
    }

    /// Prepare query for execution
    ///
    /// See [run](self::run) for more information.
    ///
    /// # Command syntax
    ///
    /// ```text
    /// query.build_query(&session) → stream
    /// query.build_query(connection) → stream
    /// query.build_query(args!(&session, options)) → stream
    /// query.build_query(args!(connection, options)) → stream
    /// query.build_query(&mut session) → stream
    /// query.build_query(args!(&mut session, options)) → stream
    /// ```
    ///
    /// Where:
    /// - session: [Session](crate::connection::Session)
    /// - connection: [Connection](crate::connection::Connection)
    /// - options: [RunOption](crate::arguments::RunOption)
    ///
    /// # Description
    ///
    /// This method has the same parameters as `run`.
    /// The main difference between `build_query` and `run` is that
    /// `build_query` can be used to execute multiple requests
    ///
    /// ## Examples
    ///
    /// You can use `query.build_query` to get the same result than `query.run`
    ///
    /// ```
    /// use futures::TryStreamExt;
    /// use neor::r;
    ///
    /// async fn example() -> neor::Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///
    ///     r.table("simbad").build_query(&conn).try_next().await?;
    ///     // is same than
    ///     r.table("simbad").run(&conn).await?;
    ///     
    ///     Ok(())
    /// }
    /// ```
    ///
    /// ## Examples
    ///
    /// Run many queries
    ///
    /// ```
    /// use futures::stream::{select_all, TryStreamExt};
    /// use neor::{r, Converter};
    ///
    /// async fn example() -> neor::Result<()> {
    ///     let conn = r.connection().connect().await?;
    ///     let mut streams = Vec::new();
    ///     let expected_messages: Vec<String> = (0..10_000)
    ///         .into_iter()
    ///         .map(|i| format!("message {}", i))
    ///         .collect();
    ///
    ///     for msg in expected_messages.iter() {
    ///         streams.push(r.expr(msg).build_query(&conn));
    ///     }
    ///
    ///     let mut list = select_all(streams);
    ///     let mut response = Vec::new();
    ///
    ///     while let Some(msg) = list.try_next().await? {
    ///         let msg: String = msg.parse()?;
    ///         response.push(msg);
    ///     }
    ///
    ///     assert!(response == expected_messages);
    ///
    ///     Ok(())
    /// }
    /// ```
    ///
    /// # Related commands
    /// - [run](self::run)
    pub fn build_query(&self, args: impl run::RunArg) -> impl Stream<Item = Result<Value>> {
        Box::pin(run::new(self.clone(), args))
    }
}

// for debug purposes only
fn bytes_to_string(bytes: &[u8]) -> String {
    if let Ok(string) = str::from_utf8(bytes) {
        return string.to_owned();
    }
    format!("{:?}", bytes)
}
