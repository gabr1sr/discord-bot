use crate::{Context, Error};
use poise::serenity_prelude::all::Emoji;
use serenity::all::{Attachment, CreateAttachment};

#[poise::command(
    slash_command,
    subcommands("show", "add"),
    subcommand_required,
    category = "Emojis"
)]
pub async fn emoji(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, required_bot_permissions = "SEND_MESSAGES", guild_only)]
pub async fn show(ctx: Context<'_>, emoji: Emoji) -> Result<(), Error> {
    ctx.reply(format!("{}?size=2048", emoji.url())).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    required_bot_permissions = "CREATE_GUILD_EXPRESSIONS",
    required_permissions = "CREATE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn add(ctx: Context<'_>, name: String, attachment: Attachment) -> Result<(), Error> {
    let bytes = attachment.download().await?;
    let builder = CreateAttachment::bytes(bytes, &name);

    let res = match ctx
        .guild_id()
        .unwrap()
        .create_emoji(&ctx, &name, &builder.to_base64())
        .await
    {
        Err(error) => format!(":x: Failed to create emoji `{name}`: {:?}", dbg!(error)),
        Ok(emoji) => format!(":white_check_mark: Emoji `{name}` created with success: {emoji}"),
    };

    ctx.reply(res).await?;
    Ok(())
}
