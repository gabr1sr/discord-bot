use crate::{Context, Error};
use serenity::builder::CreateAttachment;
use serenity::model::channel::Attachment;
use serenity::model::guild::Emoji;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("see", "add", "list", "remove"),
    subcommand_required,
    category = "Emoji"
)]
pub async fn emoji(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(ephemeral, slash_command, prefix_command, guild_only)]
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
    let data = attachment.download().await?;
    let builder = CreateAttachment::bytes(data, name.clone());
    let guild_id = ctx.guild_id().unwrap();

    let res = match guild_id
        .create_emoji(&ctx, &name, &builder.to_base64())
        .await
    {
        Err(_) => format!(":x: Failed to create emoji `{name}`"),
        Ok(emoji) => format!(":white_check_mark: Emoji created: {}", emoji),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(ephemeral, slash_command, prefix_command, guild_only)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let guild_id = ctx.guild_id().unwrap();

    let res = match guild_id.emojis(&ctx).await {
        Err(_) => format!("Failed to retrieve server emojis!"),
        Ok(emojis) => parse_emojis_list(&emojis),
    };

    ctx.reply(res).await?;
    Ok(())
}

fn parse_emojis_list(emojis: &[Emoji]) -> String {
    if emojis.is_empty() {
        return "No emojis!".to_string();
    }

    let mut lines = Vec::new();
    lines.extend(
        emojis
            .iter()
            .map(|e| format!("- <:{}:{}> `{}`", e.name, e.id.get().to_string(), e.name)),
    );
    lines.join("\n")
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, emoji: Emoji) -> Result<(), Error> {
    let res = match emoji.delete(&ctx).await {
        Err(_) => format!(":x: Failed to delete emoji `{}`", emoji.name),
        Ok(()) => format!(
            ":white_check_mark: Emoji `{}` deleted with success!",
            emoji.name
        ),
    };

    ctx.reply(res).await?;
    Ok(())
}
