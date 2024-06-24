use crate::{Context, Error};
use poise::serenity_prelude::all::Emoji;

#[poise::command(
    slash_command,
    subcommands("show"),
    subcommand_required,
    category = "Emojis"
)]
pub async fn emoji(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command, required_permissions = "SEND_MESSAGES", guild_only)]
pub async fn show(ctx: Context<'_>, emoji: Emoji) -> Result<(), Error> {
    let url = format!("{}?size=2048", emoji.url());
    ctx.reply(url).await?;
    Ok(())
}
