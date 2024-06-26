use crate::builders::CreateTag;
use crate::models::Tag;
use crate::{Context, Error};
use sqlx::{Pool, Postgres};

#[poise::command(
    slash_command,
    subcommands("add", "show"),
    subcommand_required,
    category = "Tags"
)]
pub async fn tag(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn add(ctx: Context<'_>, name: String, content: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let builder = CreateTag::new(&name, content, ctx.author());

    let res = match create_tag(&ctx.data().pool, builder).await {
        Err(error) => format!(":x: Failed to create tag `{name}`: {:?}", error),
        Ok(_) => format!(":white_check_mark: Tag `{name}` created with success!"),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(slash_command)]
pub async fn show(ctx: Context<'_>, name: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let res = match get_tag(&ctx.data().pool, &name).await {
        Err(error) => format!(":x: Failed to retrieve tag `{name}`: {:?}", error),
        Ok(tag) => tag.content,
    };

    ctx.reply(res).await?;
    Ok(())
}

pub async fn create_tag(pool: &Pool<Postgres>, builder: CreateTag) -> Result<Tag, sqlx::Error> {
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

pub async fn get_tag(pool: &Pool<Postgres>, name: impl Into<String>) -> Result<Tag, sqlx::Error> {
    sqlx::query_as!(Tag, r#"SELECT * FROM tags WHERE name = $1"#, name.into())
        .fetch_one(pool)
        .await
}
