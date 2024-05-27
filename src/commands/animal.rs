use crate::{Context, Error};

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "see"),
    subcommand_required,
    category = "Bang"
)]
pub async fn animal(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "MANAGE_GUILD",
    guild_only
)]
pub async fn add(
    ctx: Context<'_>,
    animal: String,
    emoji: String,
    points: i32,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if let Ok(_) = ctx
        .data()
        .database
        .add_animal(&animal, &emoji, points)
        .await
    {
        let res = format!(
            "New animal added: {emoji} `{animal}` which is equivalent to `{points} points!`"
        );
        ctx.reply(res).await?;
        return Ok(());
    }

    let res = format!("Failed to add new animal: `{animal}`!");
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(ephemeral, slash_command, prefix_command, guild_only)]
pub async fn see(ctx: Context<'_>, animal: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if let Ok(model) = ctx.data().database.get_animal(&animal).await {
        let emoji = model.emoji;
        let points = model.points;
        let res = format!("Animal: `{animal}`\nEmoji: `{emoji}`\nPoints: `{points}`");
        ctx.reply(res).await?;
        return Ok(());
    }

    ctx.reply(format!("Failed to retrieve animal `{animal}`!"))
        .await?;
    Ok(())
}