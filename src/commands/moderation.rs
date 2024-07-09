pub mod ban;
pub mod kick;

use crate::{Context, Error};
use poise::{
    serenity_prelude::{GetMessages, GuildChannel},
    CreateReply,
};

#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "MANAGE_MESSAGES",
    required_permissions = "MANAGE_MESSAGES",
    category = "Moderation",
    guild_only
)]
pub async fn clear(
    ctx: Context<'_>,
    #[max = 100]
    #[min = 100]
    amount: u8,
    #[rest] channel: Option<GuildChannel>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let channel = channel.unwrap_or(ctx.guild_channel().await.unwrap());
    let builder = GetMessages::new().limit(amount);

    let Ok(messages) = channel.messages(&ctx, builder).await else {
        ctx.reply(":x: Failed to retrieve messages from the channel {channel}!")
            .await?;

        return Ok(());
    };

    if messages.is_empty() {
        ctx.reply(":warning: The channel {channel} has no messages!")
            .await?;

        return Ok(());
    };

    let count = messages.len();

    let Ok(()) = channel.delete_messages(&ctx.http(), messages).await else {
        ctx.reply(":x: No messages deleted!").await?;
        return Ok(());
    };

    let res =
        format!(":white_check_mark: `{count}` messages were successfully deleted from the channel {channel}!");

    let reply = CreateReply::default().content(res).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}
