use crate::{builders::CreateTag, database::create_tag, Context, Error};

#[poise::command(
    slash_command,
    subcommands("add"),
    subcommand_required,
    category = "Tags"
)]
pub async fn tag(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(slash_command)]
pub async fn add(ctx: Context<'_>, name: String, content: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let builder = CreateTag::new(&name, content, ctx.author());

    let res = match create_tag(&ctx.data().pool, builder).await {
        Err(error) => format!(":x: Failed to create tag `{name}`: {:?}", error),
        Ok(_) => format!(":white_check_mark: Tag `{name}` created with success!"),
    };

    ctx.reply(res).await?;
    Ok(())
}
