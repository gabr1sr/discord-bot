use crate::{Context, Error};
use crate::models::TagModel;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "edit", "see"),
    subcommand_required,
    category = "Tags")]
pub async fn tag(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only
)]
pub async fn add(
    ctx: Context<'_>,
    name: String,
    content: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if let Ok(_) = sqlx::query_as!(
        TagModel,
        r#"SELECT * FROM tags WHERE name = $1"#,
        name
    )
        .fetch_one(&ctx.data().database.pool)
        .await {
            let res = format!("Tag `{name}` already exists!");
            ctx.reply(res).await?;
            return Ok(());
        }
    
    if let Ok(tag) = sqlx::query_as!(
        TagModel,
        r#"INSERT INTO tags (user_id, name, content) VALUES ($1, $2, $3) RETURNING id, user_id, name, content"#,
        ctx.author().id.to_string(),
        name,
        content
    )
        .fetch_one(&ctx.data().database.pool)
        .await {
            let res = format!("Tag `{}` created with success! ID: `{}`", tag.name, tag.id);
            ctx.reply(res).await?;
            return Ok(());
    }

    let res = format!("Cannot create tag {name}!");
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only
)]
pub async fn edit(
    ctx: Context<'_>,
    name: String,
    content: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    match sqlx::query_as!(
        TagModel,
        r#"UPDATE tags SET content = $1 WHERE user_id = $2 AND name = $3 RETURNING id, user_id, name, content"#,
        content,
        ctx.author().id.to_string(),
        name
    )
        .fetch_one(&ctx.data().database.pool)
        .await {
            Err(_) => {
                let res = format!("Tag `{name}` doesn't exists or you're not the owner of this tag!");
                ctx.reply(res).await?;
            },
            Ok(tag) => {
                let res = format!("Content of the tag `{}` updated successfully!", tag.name);
                ctx.reply(res).await?;
            }
        }
    
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only
)]
pub async fn see(
    ctx: Context<'_>,
    name: String,
) -> Result<(), Error> {
    ctx.defer().await?;

    let res =
        match sqlx::query_as!(
            TagModel,
            r#"SELECT * FROM tags WHERE name = $1"#,
            name
        )
            .fetch_one(&ctx.data().database.pool)
            .await {
                Err(_) => format!("Tag `{name}` doesn't exists!"),
                Ok(tag) => tag.content,
            };

    ctx.reply(res).await?;
    Ok(())
}
