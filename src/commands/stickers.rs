use serenity::all::{Attachment, CreateAttachment, CreateSticker, Sticker};

use crate::{Context, Error};

#[poise::command(
    slash_command,
    subcommands("show", "add"),
    subcommand_required,
    category = "Stickers"
)]
pub async fn sticker(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, guild_only)]
pub async fn show(ctx: Context<'_>, name: String) -> Result<(), Error> {
    ctx.reply(sticker_image_or_error_message(ctx, &name).await?)
        .await?;

    Ok(())
}

#[poise::command(
    slash_command,
    required_bot_permissions = "CREATE_GUILD_EXPRESSIONS",
    required_permissions = "CREATE_GUILD_EXPRESSIONS",
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
    let bytes = attachment.download().await?;
    let file = CreateAttachment::bytes(bytes, attachment.filename);
    let description = description.unwrap_or_default();

    let builder = CreateSticker::new(&name, file)
        .tags(tags)
        .description(description);

    let res = create_sticker(ctx, &name, builder).await?;
    ctx.reply(res).await?;
    Ok(())
}

async fn sticker_image_or_error_message(ctx: Context<'_>, name: &str) -> Result<String, Error> {
    let stickers: Vec<Sticker> = ctx
        .guild_id()
        .unwrap()
        .stickers(&ctx.http())
        .await?
        .into_iter()
        .filter(|s| &s.name == name)
        .collect();

    Ok(match stickers.first() {
        None => format!(":x: No sticker `{name}` found!"),
        Some(sticker) => sticker_image_url(sticker),
    })
}

fn sticker_image_url(sticker: &Sticker) -> String {
    format!("{}?size=2048", sticker.image_url().unwrap())
}

async fn create_sticker(
    ctx: Context<'_>,
    name: &str,
    builder: CreateSticker<'_>,
) -> Result<String, Error> {
    Ok(
        match ctx.guild_id().unwrap().create_sticker(&ctx, builder).await {
            Err(error) => format!(":x: Failed to create sticker `{name}`: {:?}", error),
            Ok(_) => format!(":white_check_mark: Sticker `{name}` created with success!"),
        },
    )
}
