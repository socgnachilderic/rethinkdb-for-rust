use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::Command;
use crate::Func;

use crate::cmd;
use crate::cmd::table::TableBuilder;

pub trait ReqlOpsSequence<T: Unpin + Serialize + DeserializeOwned>: SuperOps {
    /// Turn a query into a changefeed, an infinite stream of objects
    /// representing changes to the query’s results as they occur.
    /// A changefeed may return changes to a table or an individual document (a “point” changefeed).
    /// Commands such as filter or map may be used before the changes command to transform or filter the output,
    /// and many commands that operate on sequences can be chained after changes.
    fn changes(&self) -> cmd::changes::ChangesBuilder<T> {
        cmd::changes::ChangesBuilder::new()._with_parent(self.get_parent())
    }

    /// Update JSON documents in a table. Accepts a JSON document, 
    /// a ReQL expression, or a combination of the two.
    /// 
    /// You can use the following method to setting query:
    /// 
    /// * [with_durability(durability: reql_rust::types::Durability)](cmd::update::UpdateBuilder::with_durability)
    /// possible values are `Durability::Hard` and `Durability::Soft`. This option will override the table or query’s durability setting (set in [run](cmd::run)). 
    /// In soft durability mode RethinkDB will acknowledge the write immediately after receiving it, but before the write has been committed to disk.
    /// * [with_return_changes(return_changes: reql_rust::types::ReturnChanges)](cmd::update::UpdateBuilder::with_return_changes) :
    ///     - `ReturnChanges::Bool(true)` : return a `changes` array consisting of `old_val`/`new_val` objects describing the changes made, 
    ///         only including the documents actually updated.
    ///     - `ReturnChanges::Bool(false)` : do not return a `changes` array (the default).
    ///     - `ReturnChanges::Always"` : behave as `ReturnChanges::Bool(true)`, 
    ///         but include all documents the command tried to update whether or not the update was successful.
    /// * [with_non_atomic(non_atomic: bool)](cmd::update::UpdateBuilder::with_non_atomic)
    /// if set to `true`, executes the update and distributes the result to replicas in a non-atomic fashion. 
    /// This flag is required to perform non-deterministic updates, such as those that require
    /// * [with_ignore_write_hook(ignore_write_hook: bool)](cmd::update::UpdateBuilder::with_ignore_write_hook)
    /// If `true`, and if the user has the config permission, ignores any [write hook](cmd::set_write_hook::SetWriteHookBuilder) when performing the update.
    /// 
    /// Update returns a struct [WritingResponseType](crate::types::WritingResponseType):
    /// 
    /// ## Example
    /// 
    /// Update the status of all posts to published.
    /// 
    /// ```
    /// use reql_rust::{r, Result, Session};
    /// use reql_rust::prelude::*;
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let mut conn = r.connection().connect().await?;
    ///     let updated_data = json!({ "status": "published" });
    ///     
    ///     r.table("heroes").insert(&[updated_data]).run(&conn).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn update(&self, document: impl Serialize) -> cmd::update::UpdateBuilder<T> {
        cmd::update::UpdateBuilder::new(document)._with_parent(self.get_parent())
    }
    
    /// Update JSON documents in a table. Accepts a JSON document, 
    /// a ReQL expression, or a combination of the two.
    /// 
    /// See [update](#method.update) for more information
    /// 
    /// ## Example
    /// 
    /// Remove the field `status` from all posts.
    /// 
    /// ```ignore
    /// use reql_rust::{r, Result, Session};
    /// use reql_rust::prelude::*;
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let mut conn = r.connection().connect().await?;
    ///     
    ///     r.table("heroes")
    ///         .update_by_func(func!(|post| post.without("status")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn update_by_func(&self, func: Func) -> cmd::update::UpdateBuilder<T> {
        cmd::update::UpdateBuilder::new_by_func(func)._with_parent(self.get_parent())
    }

    /// Replace documents in a table. Accepts a JSON document or a ReQL expression, 
    /// and replaces the original document with the new one. 
    /// The new document must have the same primary key as the original document.
    /// 
    /// The `replace` command can be used to both insert and delete documents. 
    /// If the `“replaced”` document has a primary key that doesn’t exist in the table, 
    /// the document will be inserted; if an existing document is replaced with `None`, 
    /// the document will be deleted. Since `update`, `replace` and `replace_by_func` operations are performed atomically, 
    /// this allows atomic inserts and deletes as well.
    /// 
    /// See [update](#method.update) for more informations to setting
    /// 
    /// Replace returns a struct [WritingResponseType](crate::types::WritingResponseType):
    /// 
    /// ## Example
    /// 
    /// Remove the field `status` from all posts.
    /// 
    /// ```ignore
    /// use reql_rust::{r, Result, Session};
    /// use reql_rust::prelude::*;
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let mut conn = r.connection().connect().await?;
    ///     
    ///     r.table("heroes")
    ///         .replace(&json!({ "id": 1; "status": "published"; }))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn replace(&self, document: impl Serialize) -> cmd::replace::ReplaceBuilder<T> {
        cmd::replace::ReplaceBuilder::new(document)._with_parent(self.get_parent())
    }

    /// Replace documents in a table. Accepts a JSON document or a ReQL expression, 
    /// and replaces the original document with the new one. 
    /// 
    /// See [replace](#method.replace) for more information
    /// 
    /// ## Example
    /// 
    /// Remove the field `status` from all posts.
    /// 
    /// ```ignore
    /// use reql_rust::{r, Result, Session};
    /// use reql_rust::prelude::*;
    /// use serde_json::json;
    ///
    /// async fn example() -> Result<()> {
    ///     let mut conn = r.connection().connect().await?;
    ///     
    ///     r.table("heroes")
    ///         .replace_by_func(func!(|post| post.without("status")))
    ///         .run(&conn)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn replace_by_func(&self, func: Func) -> cmd::replace::ReplaceBuilder<T> {
        cmd::replace::ReplaceBuilder::new_by_func(func)._with_parent(self.get_parent())
    }

    /// Delete one or more documents from a table.
    /// 
    /// You can use the following method to setting query:
    /// 
    /// * [with_durability(durability: reql_rust::types::Durability)](cmd::update::UpdateBuilder::with_durability)
    /// possible values are `Durability::Hard` and `Durability::Soft`. This option will override the table or query’s durability setting (set in [run](cmd::run)). 
    /// In soft durability mode RethinkDB will acknowledge the write immediately after receiving it, but before the write has been committed to disk.
    /// * [with_return_changes(return_changes: reql_rust::types::ReturnChanges)](cmd::update::UpdateBuilder::with_return_changes) :
    ///     - `ReturnChanges::Bool(true)` : return a `changes` array consisting of `old_val`/`new_val` objects describing the changes made, 
    ///         only including the documents actually updated.
    ///     - `ReturnChanges::Bool(false)` : do not return a `changes` array (the default).
    ///     - `ReturnChanges::Always"` : behave as `ReturnChanges::Bool(true)`, 
    ///         but include all documents the command tried to update whether or not the update was successful.
    /// * [with_ignore_write_hook(ignore_write_hook: bool)](cmd::update::UpdateBuilder::with_ignore_write_hook)
    /// If `true`, and if the user has the config permission, ignores any [write hook](cmd::set_write_hook::SetWriteHookBuilder), 
    /// which might have prohibited the deletion.
    /// 
    /// ## Example
    /// 
    /// Delete a single document from the table `heroes` .
    /// 
    /// ```
    /// use reql_rust::{r, Result, Session};
    /// use reql_rust::prelude::*;
    /// use serde::{Serialize, Deserialize};
    /// use serde_json::json;
    /// 
    /// #[derive(Serialize, Deserialize)]
    /// struct Heroes {
    ///     id: String,
    ///     name: String,
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let mut conn = r.connection().connect().await?;
    ///     
    ///     r.table::<Heroes>("heroes").delete().run(&conn).await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn delete(&self) -> cmd::delete::DeleteBuilder<T> {
        cmd::delete::DeleteBuilder::new()._with_parent(self.get_parent())
    }

    /// Return all the elements in a sequence for which the given predicate is true.
    /// The return value of `filter` will be the same as the input (sequence, stream, or array).
    /// Documents can be filtered in a variety of ways—ranges, nested values, boolean conditions,
    /// and the results of anonymous functions.
    fn filter(&self, func: Func) -> cmd::filter::FilterBuilder<T> {
        cmd::filter::FilterBuilder::new(func)._with_parent(self.get_parent())
    }
    
    /// Returns an inner join of two sequences.
    ///
    /// The returned sequence represents an intersection of the left-hand sequence and the right-hand sequence:
    /// each row of the left-hand sequence will be compared with
    /// each row of the right-hand sequence  to find all pairs of rows which satisfy the predicate.
    /// Each matched pair of rows of both sequences are combined  into a result row.
    /// In most cases, you will want to follow the join with [zip](self::ReqlOpsJoin::zip) to combine the left and right results.
    ///
    /// ```text
    /// Note that inner_join is slower and much less efficient than using eq_join or concat_map with get_all.
    /// You should avoid using inner_join in commands when possible.
    /// ```
    ///
    /// ## Example
    ///
    /// Return a list of all matchups between Marvel and DC heroes in which the DC hero could beat the Marvel hero in a fight.
    ///
    /// ```
    /// use reql_rust::prelude::*;
    /// use reql_rust::{r, Result};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Users {
    ///     id: u8,
    ///     full_name: String,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Posts {
    ///     id: u8,
    ///     title: String,
    ///     content: String,
    ///     user_id: u8,
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let session = r.connection().connect().await?;
    ///     let _ = r.table::<Posts>("posts")
    ///         .inner_join(
    ///             &r.table::<Users>("users"),
    ///             func!(|post, _user| post.bracket("user_id").eq(1)),
    ///         )
    ///         .run(&session)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn inner_join<A: Unpin + Serialize + DeserializeOwned>(
        &self,
        other_table: &TableBuilder<A>,
        func: Func,
    ) -> cmd::inner_join::InnerJoinBuilder<A, T> {
        cmd::inner_join::InnerJoinBuilder::new(other_table, func)._with_parent(self.get_parent())
    }

    /// Returns a left outer join of two sequences.
    /// The returned sequence represents a union of the left-hand sequence and the right-hand sequence:
    /// all documents in the left-hand sequence will be returned,
    /// each matched with a document in the right-hand sequence if one satisfies the predicate condition.
    /// In most cases, you will want to follow the join with [zip](self::ReqlOpsJoin::zip) to combine the left and right results.
    ///
    /// ```
    /// use reql_rust::prelude::*;
    /// use reql_rust::{r, Result};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Users {
    ///     id: u8,
    ///     full_name: String,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Posts {
    ///     id: u8,
    ///     title: String,
    ///     content: String,
    ///     user_id: u8,
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let session = r.connection().connect().await?;
    ///     let _ = r.table::<Posts>("posts")
    ///         .outer_join(
    ///             &r.table::<Users>("users"),
    ///             func!(|post, _user| post.bracket("user_id").eq(1)),
    ///         )
    ///         .run(&session)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn outer_join<A: Unpin + Serialize + DeserializeOwned>(
        &self,
        other_table: &TableBuilder<A>,
        func: Func,
    ) -> cmd::outer_join::OuterJoinBuilder<A, T> {
        cmd::outer_join::OuterJoinBuilder::new(other_table, func)._with_parent(self.get_parent())
    }

    /// Join tables using a field or function on the left-hand sequence matching primary keys or secondary indexes on the right-hand table. 
    /// `eq_join` is more efficient than other ReQL join types, and operates much faster. 
    /// Documents in the result set consist of pairs of left-hand and right-hand documents, 
    /// matched when the field on the left-hand side exists and is non-null and an entry 
    /// with that field’s value exists in the specified index on the right-hand side.
    /// 
    /// The result set of `eq_join` is a stream or array of objects. 
    /// Each object in the returned set will be an object of the form { "left": <left-document>, "right": <right-document> }, 
    /// where the values of left and right will be the joined documents. 
    /// Use the [zip](self::ReqlOpsJoin::zip) command to merge the left and right fields together.
    /// 
    /// The results from `eq_join` are, by default, not ordered. Providing [with_ordered(true)](cmd::eq_join::EqJoinBuilder::with_ordered) 
    /// will cause `eq_join` to order the output based on the left side input stream. 
    /// (If there are multiple matches on the right side for a document on the left side, 
    /// their order is not guaranteed even if ordered is true.) Requiring ordered results can significantly slow down `eq_join`, 
    /// and in many circumstances this ordering will not be required. 
    /// (See the first example, in which ordered results are obtained by using `order_by` after `eq_join`.)
    /// 
    /// ## Example
    /// 
    /// Match posts with the users they’ve posted against one another.
    /// 
    /// Join these tables using `user_id` on the users table and `id` on the posts table:
    /// 
    /// ```
    /// use reql_rust::prelude::*;
    /// use reql_rust::{r, Result};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Users {
    ///     id: u8,
    ///     full_name: String,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Posts {
    ///     id: u8,
    ///     title: String,
    ///     content: String,
    ///     user_id: u8,
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let session = r.connection().connect().await?;
    ///     let _ = r.table::<Posts>("posts")
    ///         .eq_join(
    ///             "user_id",
    ///             &r.table::<Users>("users"),
    ///         )
    ///         .run(&session)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn eq_join<A: Unpin + Serialize + DeserializeOwned>(
        &self,
        left_field: &str,
        right_table: &TableBuilder<A>,
    ) -> cmd::eq_join::EqJoinBuilder<A, T> {
        cmd::eq_join::EqJoinBuilder::new(left_field, right_table)._with_parent(self.get_parent())
    }

    /// Join tables using a field or function on the left-hand sequence matching primary keys or secondary indexes on the right-hand table. 
    /// `eq_join` is more efficient than other ReQL join types, and operates much faster. 
    /// Documents in the result set consist of pairs of left-hand and right-hand documents, 
    /// matched when the field on the left-hand side exists and is non-null and an entry 
    /// with that field’s value exists in the specified index on the right-hand side.
    /// 
    /// See [eq_join](#method.eq_join) for more informations
    /// 
    /// ## Example
    /// 
    /// Match posts with the users they’ve posted against one another.
    /// 
    /// Join these tables using `user_id` on the users table and `id` on the posts table:
    /// 
    /// ```
    /// use reql_rust::prelude::*;
    /// use reql_rust::{r, Result};
    /// use serde::{Serialize, Deserialize};
    ///
    /// #[derive(Debug, Serialize, Deserialize)]
    /// struct Users {
    ///     id: u8,
    ///     full_name: String,
    /// }
    ///
    /// #[derive(Serialize, Deserialize, Debug)]
    /// struct Posts {
    ///     id: u8,
    ///     title: String,
    ///     content: String,
    ///     user_id: u8,
    /// }
    ///
    /// async fn example() -> Result<()> {
    ///     let session = r.connection().connect().await?;
    ///     let _ = r.table::<Posts>("posts")
    ///         .eq_join_by_func(
    ///             func!(|row| row.bracket("user_id")),
    ///             &r.table::<Users>("users"),
    ///         )
    ///         .run(&session)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn eq_join_by_func<A: Unpin + Serialize + DeserializeOwned>(
        &self,
        func: Func,
        right_table: &TableBuilder<A>,
    ) -> cmd::eq_join::EqJoinBuilder<A, T> {
        cmd::eq_join::EqJoinBuilder::new_by_func(func, right_table)._with_parent(self.get_parent())
    }
}

pub trait ReqlOpsArray: SuperOps {
    
}

pub trait ReqlOpsJoin: SuperOps {
    /// Used to ‘zip’ up the result of a join by merging the ‘right’ fields into ‘left’ fields of each member of the sequence.
    /// 
    /// ## Example
    /// 
    /// ‘zips up’ the sequence by merging the left and right fields produced by a join.
    /// 
    /// ```
    /// use reql_rust::prelude::*;
    /// use reql_rust::{r, Result};
    /// use serde::{Serialize, Deserialize};
    /// use serde_json::Value;
    ///
    /// async fn example() -> Result<()> {
    ///     let session = r.connection().connect().await?;
    ///     let _ = r.table::<Value>("marvel")
    ///         .eq_join(
    ///             "main_dc_collaborator",
    ///             &r.table::<Value>("dc"),
    ///         )
    ///         .zip()
    ///         .run(&session)
    ///         .await?;
    ///
    ///     Ok(())
    /// }
    /// ```
    fn zip(&self) -> cmd::zip::ZipBuilder {
        cmd::zip::ZipBuilder::new()._with_parent(self.get_parent())
    }
}

pub trait SuperOps {
    fn get_parent(&self) -> Command;
}
