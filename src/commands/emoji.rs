use crate::{Context, Error};
use poise::samples::paginate;
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

    if let Ok(emojis) = guild_id.emojis(&ctx).await {
        if emojis.is_empty() {
            ctx.reply(":warning: Server has no emojis!").await?;
            return Ok(());
        }

        let emojis_vec = emojis
            .into_iter()
            .map(|e| format!("- <:{}:{}> `{}`\n", e.name, e.id.get().to_string(), e.name))
            .collect::<Vec<_>>();

        let chunks = emojis_vec
            .chunks(10)
            .map(|c| c.join("\n"))
            .collect::<Vec<_>>();

        let pages: Vec<&str> = chunks.iter().map(|s| s.as_ref()).collect();

        paginate(ctx, &pages).await?;
        return Ok(());
    }

    ctx.reply(":x: Failed to retrieve server emojis!").await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, emoji: Emoji) -> Result<(), Error> {
    let guild_id = ctx.guild_id().unwrap();

    let res = match guild_id.delete_emoji(&ctx, &emoji).await {
        Err(_) => format!(":x: Failed to delete emoji `{}`", emoji.name),
        Ok(()) => format!(
            ":white_check_mark: Emoji `{}` deleted with success!",
            emoji.name
        ),
    };

    ctx.reply(res).await?;
    Ok(())
}
