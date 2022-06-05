use reql_rust::prelude::*;
use reql_rust::types::Interleave;
use reql_rust::{r, Result, Session};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Users {
    id: u8,
    full_name: String,
    posts: [u8; 2],
}

#[derive(Serialize, Deserialize, Debug)]
struct Posts {
    id: u8,
    title: String,
    content: String,
    note: f32,
    user_id: u8,
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut conn = r.connection().connect().await?;
    set_up(&conn).await?;
    conn.use_("marvel").await;

    let user_table = r.db("marvel").table::<Users>("users");
    let post_table = r.db("marvel").table::<Posts>("posts");

    let result = post_table.run(&conn).await?;
    dbg!(result);

    let result = post_table.get(2).run(&conn).await?;
    dbg!(result);

    let result = post_table
        .get_all(&[1, 2])
        .with_index("id")
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table.between(1, 4).run(&conn).await?;
    dbg!(result);

    // let result = post_table
    //     .filter(func!(|row| row.bracket("id").eq(3)))
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    // let result = post_table
    //     .inner_join(
    //         &user_table,
    //         func!(|post, _user| post.bracket("user_id").eq(1)),
    //     )
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    // let result = post_table
    //     .outer_join(
    //         &user_table,
    //         func!(|post, _user| post.bracket("user_id").eq(1)),
    //     )
    //     .zip()
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    let result = post_table
        .eq_join("user_id", &user_table)
        .with_ordered(true)
        .run(&conn)
        .await?;
    dbg!(result);

    // let result = post_table
    //     .map::<String>(func!(|row| row.bracket("title")))
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    #[derive(Debug, Serialize, Deserialize)]
    struct NewPost {
        id: u8,
        title: String,
    }

    let result = post_table
        .with_fields::<NewPost>(&["id", "title"])
        .run(&conn)
        .await?;
    dbg!(result);

    // let result = user_table
    //     .concat_map::<u8>(func!(|row| row.bracket("posts")))
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    let result = user_table
        .order_by_key("full_name")
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .skip(3)
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .limit(3)
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .slice(2, None)
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .nth(-1)
        .run(&conn)
        .await?;
    dbg!(result);

    // let result = post_table
    //     .offsets_of(-1)
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    let result = post_table
        .is_empty()
        .run(&conn)
        .await?;
    dbg!(result);

    
    #[derive(Serialize, Deserialize, Debug)]
    struct MergePostAndUser {
        id: u8,
        full_name: Option<String>,
        posts: Option<[u8; 2]>,
        title: Option<String>,
        content: Option<String>,
        note: Option<f32>,
        user_id: Option<u8>,
    }
    
    let result = post_table
        .union::<_, MergePostAndUser>(&[&user_table])
        .with_interleave(Interleave::Bool(false))
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .sample(3)
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .group::<u8>(&["user_id"])
        // .with_index("title")
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .group::<u8>(&["user_id"])
        .ungroup()
        .sample(1)
        .run(&conn)
        .await?;
    dbg!(result);

    // let result = post_table
    //     .reduce::<serde_json::Value>(func!(|left, right| left.bracket("title")))
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    let result = post_table.count().run(&conn).await?;
    dbg!(result);

    let result = post_table.sum_by_field("note").run(&conn).await?;
    dbg!(result);

    let result = post_table.avg_by_field("note").run(&conn).await?;
    dbg!(result);

    let result = post_table.min_by_field("note").run(&conn).await?;
    dbg!(result);

    let result = post_table.max_by_field("note").run(&conn).await?;
    dbg!(result);

    let result = post_table.distinct().run(&conn).await?;
    dbg!(result);

    // let result = post_table
    //     .fold::<_, >(0, func!(|acc, row| r.))
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    // let result = post_table
    //     .contains("title")
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    #[derive(Debug, Serialize, Deserialize)]
    struct PostsProjection {
        title: String,
        content: String,
    }

    let result = post_table
        .pluck::<_, Vec<PostsProjection>>(["title", "content"])
        .run(&conn)
        .await?;
    dbg!(result);

    #[derive(Debug, Serialize, Deserialize)]
    struct PostWithout {
        id: u8,
        note: f32,
        title: String,
    }

    let result = post_table
        .without::<_, Vec<PostWithout>>(["user_id", "content"])
        .run(&conn)
        .await?;
    dbg!(result);

    // let result = post_table
    //     .get(1)
    //     .bracket("title")
    //     .append::<_, serde_json::Value>("title 1")
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    // let result = post_table
    //     .get(1)
    //     .bracket("title")
    //     .difference::<_, serde_json::Value>(&["title 1"])
    //     .run(&conn)
    //     .await?;
    // dbg!(result);

    let result: String = post_table
        .get(1)
        .bracket("title")
        .run(&conn)
        .await?
        .unwrap()
        .parse();
    dbg!(result);

    let result = post_table
        .get(1)
        .get_field("title")
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .has_fields("title")
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .get(1)
        .keys()
        .run(&conn)
        .await?;
    dbg!(result);

    let result = post_table
        .get(1)
        .values()
        .run(&conn)
        .await?;
    dbg!(result);

    tear_down(&conn).await?;

    Ok(())
}

async fn set_up(conn: &Session) -> Result<()> {
    let users = vec![
        Users {
            id: 1,
            full_name: "John Doe".to_string(),
            posts: [1, 2]
        },
        Users {
            id: 2,
            full_name: "Don Juan".to_string(),
            posts: [3, 5]
        },
    ];

    let posts = vec![
        Posts {
            id: 1,
            title: "title 1".to_string(),
            content: "content 1".to_string(),
            note: 4.5,
            user_id: 1,
        },
        Posts {
            id: 2,
            title: "title 2".to_string(),
            content: "content 2".to_string(),
            note: 2.5,
            user_id: 2,
        },
        Posts {
            id: 3,
            title: "title 3".to_string(),
            content: "content 3".to_string(),
            note: 5.,
            user_id: 1,
        },
        Posts {
            id: 4,
            title: "title 4".to_string(),
            content: "content 4".to_string(),
            note: 4.,
            user_id: 2,
        },
        Posts {
            id: 5,
            title: "title 5".to_string(),
            content: "content 5".to_string(),
            note: 3.5,
            user_id: 1,
        },
    ];

    r.db_create("marvel").run(conn).await?;
    r.db("marvel").table_create("users").run(conn).await?;
    r.db("marvel").table_create("posts").run(conn).await?;

    r.db("marvel")
        .table::<Posts>("posts")
        .index_create("title")
        .run(conn)
        .await?;

    r.db("marvel")
        .table("users")
        .insert(&users)
        .run(conn)
        .await?;
    r.db("marvel")
        .table("posts")
        .insert(&posts)
        .run(conn)
        .await?;

    Ok(())
}

async fn tear_down(conn: &Session) -> Result<()> {
    r.db_drop("marvel").run(conn).await?;

    Ok(())
}