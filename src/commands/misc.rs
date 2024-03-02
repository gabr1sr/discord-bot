use crate::{Context, Error};

use crate::translation::tr;

#[poise::command(slash_command, prefix_command, category = "Misc")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let res = format!("{}", tr!(ctx, "Pong"));
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Misc")]
pub async fn database(ctx: Context<'_>) -> Result<(), Error> {
    let rows = ctx
        .data()
        .database
        .client
        .query("SELECT $1::TEXT", &[&"hello world"])
        .await?;

    let value: &str = rows[0].get(0);
    ctx.reply(value.to_string()).await?;
    Ok(())
}
