use crate::{utils::emoji_identifiers_from, Context, Error};
use poise::serenity_prelude::all::{Attachment, CreateAttachment, Emoji, Message};

#[poise::command(
    slash_command,
    subcommands("show", "add", "remove"),
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
    let res = create_emoji(ctx, &name, &builder.to_base64()).await?;
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    required_bot_permissions = "MANAGE_GUILD_EXPRESSIONS",
    required_permissions = "MANAGE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, emoji: Emoji) -> Result<(), Error> {
    let name = &emoji.name;

    let res = match ctx.guild_id().unwrap().delete_emoji(&ctx, &emoji).await {
        Err(error) => format!(":x: Failed to delete emoji `{name}`: {:?}", dbg!(error)),
        Ok(()) => format!(":white_check_mark: Emoji `{name}` deleted with success!"),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(context_menu_command = "Retrieve emoji image", category = "Emojis")]
pub async fn retrieve_emoji_context(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let emojis = emoji_identifiers_from(&message.content);

    let res = match emojis.first() {
        Some(emoji) => emoji.url(),
        None => ":x: Failed to retrieve any emoji from the message!".to_owned(),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "Clone emoji",
    required_bot_permissions = "CREATE_GUILD_EXPRESSIONS",
    required_permissions = "CREATE_GUILD_EXPRESSIONS",
    category = "Emojis",
    guild_only
)]
pub async fn clone_emoji_context(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let emojis = emoji_identifiers_from(&message.content);

    let Some(emoji) = emojis.first() else {
        ctx.reply(":x: Failed to retrieve any emoji from the message!")
            .await?;

        return Ok(());
    };

    let name = &emoji.name;
    let url = emoji.url();
    let builder = CreateAttachment::url(ctx.http(), &url).await?;
    let res = create_emoji(ctx, &name, &builder.to_base64()).await?;
    ctx.reply(res).await?;
    Ok(())
}

async fn create_emoji(ctx: Context<'_>, name: &str, image: &str) -> Result<String, Error> {
    Ok(
        match ctx
            .guild_id()
            .unwrap()
            .create_emoji(&ctx, name, image)
            .await
        {
            Err(error) => format!(":x: Failed to create emoji `{name}`: {:?}", dbg!(error)),
            Ok(emoji) => format!(":white_check_mark: Emoji `{name}` created with success: {emoji}"),
        },
    )
}
