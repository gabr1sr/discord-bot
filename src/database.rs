use crate::builders::CreateTag;
use crate::models::Tag;
use sqlx::{Error, Pool, Postgres};

pub async fn create_tag(pool: &Pool<Postgres>, builder: CreateTag) -> Result<Tag, Error> {
    sqlx::query_as!(
        Tag,
        r#"INSERT INTO tags (name, content, owner) VALUES ($1, $2, $3) RETURNING id, name, content, owner"#,
        builder.name,
        builder.content,
        builder.owner.get().to_string()
    )
        .fetch_one(pool)
        .await
}

pub async fn get_tag(pool: &Pool<Postgres>, name: impl Into<String>) -> Result<Tag, Error> {
    sqlx::query_as!(Tag, r#"SELECT * FROM tags WHERE name = $1"#, name.into())
        .fetch_one(pool)
        .await
}
