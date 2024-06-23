use serenity::all::{Attachment, CreateAttachment, CreateSticker};

use crate::{Context, Error};

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add"),
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
