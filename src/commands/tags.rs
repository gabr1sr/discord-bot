use crate::builders::CreateTag;
use crate::models::Tag;
use crate::{Context, Error};
use poise::{
    serenity_prelude as serenity, serenity_prelude::futures::TryStreamExt, serenity_prelude::UserId,
};
use sqlx::{Pool, Postgres};
use std::collections::HashMap;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "show", "edit", "remove", "list"),
    subcommand_required,
    category = "Tags"
)]
pub async fn tag(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn add(ctx: Context<'_>, name: String, #[rest] content: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let builder = CreateTag::new(&name, content, ctx.author());

    let res = match create_tag(&ctx.data().pool, builder).await {
        Err(error) => format!(":x: Failed to create tag `{name}`: {:?}", error),
        Ok(_) => format!(":white_check_mark: Tag `{name}` created with success!"),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn show(ctx: Context<'_>, #[rest] name: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let res = match get_tag(&ctx.data().pool, &name).await {
        Err(error) => format!(":x: Failed to retrieve tag `{name}`: {:?}", error),
        Ok(tag) => tag.content,
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn edit(ctx: Context<'_>, name: String, #[rest] content: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let builder = CreateTag::new(&name, content, ctx.author());

    let res = match update_tag(&ctx.data().pool, builder).await {
        Err(_) => format!(":x: You're not the owner of the tag `{name}` or it doesn't exist!"),
        Ok(_) => format!(":white_check_mark: Tag `{name}` updated with success!"),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn remove(ctx: Context<'_>, #[rest] name: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let builder = CreateTag::new(&name, "", ctx.author());

    let res = match delete_tag(&ctx.data().pool, builder).await {
        Err(_) | Ok(0) => {
            format!(":x: You're not the owner of the tag `{name}` or it doesn't exist!")
        }
        Ok(_) => format!(":white_check_mark: Tag `{name}` deleted with success!"),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
pub async fn list(ctx: Context<'_>, owner: Option<UserId>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let rows = match owner {
        None => get_all_tags(&ctx.data().pool).await,
        Some(owner) => get_owner_tags(&ctx.data().pool, owner).await,
    };

    paginate_tags(ctx, rows).await?;
    Ok(())
}

async fn paginate_tags(
    ctx: Context<'_>,
    tags: std::pin::Pin<
        Box<
            dyn poise::futures_util::Stream<Item = Result<Tag, sqlx::Error>>
                + std::marker::Send
                + '_,
        >,
    >,
) -> Result<(), Error> {
    let ctx_id = ctx.id();
    let prev_button_id = format!("{}prev", ctx_id);
    let next_button_id = format!("{}next", ctx_id);

    let mut chunks = tags.try_chunks(10);
    let mut map: HashMap<u32, String> = HashMap::new();
    let mut current_page: u32 = 0;

    let first_content = format_tags(chunks.try_next().await?);

    if first_content.is_empty() {
        ctx.reply(":x: No tags found!").await?;
        return Ok(());
    }

    let reply = {
        let components = serenity::CreateActionRow::Buttons(vec![
            serenity::CreateButton::new(&prev_button_id).emoji('◀'),
            serenity::CreateButton::new(&next_button_id).emoji('▶'),
        ]);

        map.insert(current_page, first_content.clone());

        let embed = serenity::CreateEmbed::default()
            .title("Tags List")
            .description(first_content);

        poise::CreateReply::default()
            .embed(embed)
            .components(vec![components])
    };

    ctx.send(reply).await?;

    while let Some(press) = serenity::collector::ComponentInteractionCollector::new(ctx)
        .filter(move |press| press.data.custom_id.starts_with(&ctx_id.to_string()))
        .timeout(std::time::Duration::from_secs(60))
        .await
    {
        if press.data.custom_id == next_button_id {
            current_page += 1;

            if !map.contains_key(&current_page) {
                let content = format_tags(chunks.try_next().await?);

                if !content.is_empty() {
                    map.insert(current_page, content.clone());
                } else {
                    current_page = 0;
                }
            }
        } else if press.data.custom_id == prev_button_id {
            current_page = current_page.checked_sub(1).unwrap_or_default();
        } else {
            continue;
        }

        let content = map.get(&current_page).unwrap();

        let embed = serenity::CreateEmbed::default()
            .title("Tags List")
            .description(content);

        press
            .create_response(
                ctx.serenity_context(),
                serenity::CreateInteractionResponse::UpdateMessage(
                    serenity::CreateInteractionResponseMessage::new().embed(embed),
                ),
            )
            .await?;
    }

    Ok(())
}

fn format_tags(tags: Option<Vec<Tag>>) -> String {
    let Some(tags) = tags else {
        return String::new();
    };

    tags.into_iter()
        .map(|t| format!("- {}", t.name))
        .collect::<Vec<_>>()
        .join("\n")
}

async fn create_tag(pool: &Pool<Postgres>, builder: CreateTag) -> Result<Tag, sqlx::Error> {
    sqlx::query_as!(
        Tag,
        r#"INSERT INTO tags (name, content, owner) VALUES ($1, $2, $3) RETURNING *"#,
        builder.name,
        builder.content,
        builder.owner.get().to_string()
    )
    .fetch_one(pool)
    .await
}

async fn get_tag(pool: &Pool<Postgres>, name: impl Into<String>) -> Result<Tag, sqlx::Error> {
    sqlx::query_as!(Tag, r#"SELECT * FROM tags WHERE name = $1"#, name.into())
        .fetch_one(pool)
        .await
}

async fn get_all_tags(
    pool: &Pool<Postgres>,
) -> std::pin::Pin<
    Box<dyn poise::futures_util::Stream<Item = Result<Tag, sqlx::Error>> + std::marker::Send + '_>,
> {
    sqlx::query_as!(Tag, r#"SELECT * FROM tags"#).fetch(pool)
}

async fn get_owner_tags(
    pool: &Pool<Postgres>,
    owner: impl Into<u64>,
) -> std::pin::Pin<
    Box<dyn poise::futures_util::Stream<Item = Result<Tag, sqlx::Error>> + std::marker::Send + '_>,
> {
    sqlx::query_as!(
        Tag,
        r#"SELECT * FROM tags WHERE owner = $1"#,
        owner.into().to_string()
    )
    .fetch(pool)
}

async fn update_tag(pool: &Pool<Postgres>, builder: CreateTag) -> Result<Tag, sqlx::Error> {
    sqlx::query_as!(
        Tag,
        r#"UPDATE tags SET content = $1 WHERE name = $2 AND owner = $3 RETURNING *"#,
        builder.content,
        builder.name,
        builder.owner.get().to_string()
    )
    .fetch_one(pool)
    .await
}

async fn delete_tag(pool: &Pool<Postgres>, builder: CreateTag) -> Result<u64, sqlx::Error> {
    Ok(sqlx::query!(
        r#"DELETE FROM tags WHERE name = $1 AND owner = $2"#,
        builder.name,
        builder.owner.get().to_string()
    )
    .execute(pool)
    .await?
    .rows_affected())
}
