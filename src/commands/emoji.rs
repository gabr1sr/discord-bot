use crate::{Context, Error};
use serenity::model::guild::Emoji;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("see"),
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
