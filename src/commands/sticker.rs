use serenity::all::{Attachment, CreateAttachment, CreateSticker, Sticker};

use crate::{Context, Error};

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "show"),
    subcommand_required,
    category = "Sticker"
)]
pub async fn sticker(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "CREATE_GUILD_EXPRESSIONS",
    required_bot_permissions = "CREATE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn add(
    ctx: Context<'_>,
    name: String,
    tags: String,
    attachment: Attachment,
    description: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let data = attachment.download().await?;
    let file = CreateAttachment::bytes(data, attachment.filename);
    let description = description.unwrap_or("".to_owned());

    let builder = CreateSticker::new(&name, file)
        .tags(tags)
        .description(description);

    let guild_id = ctx.guild_id().unwrap();

    let res = match guild_id.create_sticker(&ctx, builder).await {
        Err(_) => format!(":x: Failed to create sticker `{name}`!"),
        Ok(sticker) => format!(
            ":white_check_mark: Sticker `{}` created with success!",
            sticker.name
        ),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(ephemeral, slash_command, prefix_command, guild_only)]
pub async fn show(
    ctx: Context<'_>,
    name: Option<String>,
    tags: Option<String>,
) -> Result<(), Error> {
    let name = name.unwrap_or("".to_owned());
    let tags = tags.unwrap_or("".to_owned());

    if name.is_empty() && tags.is_empty() || !name.is_empty() && !tags.is_empty() {
        ctx.reply(":warning: Must provide `name` or `tags`!")
            .await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let stickers = guild_id.stickers(&ctx.http()).await?;

    if stickers.is_empty() {
        ctx.reply(":x: Guild has no stickers!").await?;
        return Ok(());
    }

    let filtered: Vec<&Sticker> = if tags.is_empty() {
        stickers.iter().filter(|s| &s.name == &name).collect()
    } else {
        stickers.iter().filter(|s| s.tags.contains(&tags)).collect()
    };

    let sticker = filtered.first();

    if let None = sticker {
        ctx.reply(":x: No sticker found!").await?;
        return Ok(());
    }

    let sticker = sticker.unwrap();
    ctx.reply(sticker.image_url().unwrap()).await?;
    Ok(())
}
