use serenity::all::{Attachment, CreateAttachment, CreateSticker, Message, Sticker, StickerItem};

use crate::{Context, Error};

#[poise::command(
    slash_command,
    subcommands("show", "add", "remove"),
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

#[poise::command(
    slash_command,
    required_bot_permissions = "MANAGE_GUILD_EXPRESSIONS",
    required_permissions = "MANAGE_GUILD_EXPRESSIONS",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, name: String) -> Result<(), Error> {
    let res = match get_sticker(ctx, &name).await? {
        None => ":x: No sticker `{name}` found!".to_owned(),
        Some(sticker) => delete_sticker(ctx, sticker).await?,
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(context_menu_command = "Retrieve sticker image", category = "Stickers")]
pub async fn retrieve_sticker_context(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let res = match message.sticker_items.first() {
        None => ":x: Failed to retrieve sticker from message!".to_owned(),
        Some(sticker) => sticker_item_image_url(sticker),
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    context_menu_command = "Clone sticker",
    required_bot_permissions = "CREATE_GUILD_EXPRESSIONS",
    required_permissions = "CREATE_GUILD_EXPRESSIONS",
    category = "Stickers",
    guild_only
)]
pub async fn clone_sticker_context(ctx: Context<'_>, message: Message) -> Result<(), Error> {
    let Some(sticker_item) = message.sticker_items.first() else {
        ctx.reply(":x: Failed to retrieve sticker from message!".to_owned())
            .await?;

        return Ok(());
    };

    let image_url = sticker_item_image_url(sticker_item);
    let sticker = sticker_item.to_sticker(&ctx.http()).await?;
    let attachment = CreateAttachment::url(&ctx.http(), &image_url).await?;

    let builder = CreateSticker::new(&sticker.name, attachment)
        .tags(sticker.tags.join(", "))
        .description(sticker.description.unwrap_or_default());

    ctx.reply(create_sticker(ctx, &sticker.name, builder).await?)
        .await?;

    Ok(())
}

async fn get_sticker(ctx: Context<'_>, name: &str) -> Result<Option<Sticker>, Error> {
    let stickers: Vec<Sticker> = ctx
        .guild_id()
        .unwrap()
        .stickers(&ctx.http())
        .await?
        .into_iter()
        .filter(|s| &s.name == name)
        .collect();

    if let Some(sticker) = stickers.first() {
        return Ok(Some(sticker.clone()));
    }

    Ok(None)
}

async fn sticker_image_or_error_message(ctx: Context<'_>, name: &str) -> Result<String, Error> {
    Ok(match get_sticker(ctx, name).await? {
        None => format!(":x: No sticker `{name}` found!"),
        Some(sticker) => sticker_image_url(&sticker),
    })
}

fn sticker_image_url(sticker: &Sticker) -> String {
    format!("{}?size=2048", sticker.image_url().unwrap())
}

fn sticker_item_image_url(sticker_item: &StickerItem) -> String {
    format!("{}?size=2048", sticker_item.image_url().unwrap())
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

async fn delete_sticker(ctx: Context<'_>, sticker: Sticker) -> Result<String, Error> {
    let name = &sticker.name;

    Ok(
        match ctx
            .guild_id()
            .unwrap()
            .delete_sticker(&ctx.http(), sticker.id)
            .await
        {
            Err(error) => format!(":x: Failed to delete sticker `{name}`: {:?}", error),
            Ok(_) => format!(":white_check_mark: Sticker `{name}` deleted with success!"),
        },
    )
}
