use crate::{Context, Error};
use serenity::model::guild::Emoji;
use serenity::model::channel::Attachment;
use serenity::builder::CreateAttachment;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("see", "add"),
    subcommand_required,
    category = "Emoji"
)]
pub async fn emoji(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only
)]
pub async fn see(ctx: Context<'_>, emoji: Emoji) -> Result<(), Error> {
    ctx.reply(emoji.url()).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "CREATE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn add(ctx: Context<'_>, name: String, attachment: Attachment) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let data = attachment.download().await?;
    let builder = CreateAttachment::bytes(data, name.clone());
    
    let res =
        match guild_id.create_emoji(&ctx, &name, &builder.to_base64()).await {
            Err(_) => format!("Failed to create emoji `{name}`"),
            Ok(emoji) => format!("Emoji created: {}", emoji),
        };

    ctx.reply(res).await?;
    Ok(())
}
