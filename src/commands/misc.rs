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
    let row: (i64,) = sqlx::query_as("SELECT $1")
        .bind(150_i64)
        .fetch_one(&ctx.data().database.pool).await?;

    let res = format!("Value: {}", row.0);
    ctx.reply(res).await?;
    Ok(())
}
