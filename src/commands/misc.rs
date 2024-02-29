use crate::{Context, Error};

use crate::translation::tr;

#[poise::command(slash_command, prefix_command, category = "Misc")]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    let res = format!("{}", tr!(ctx, "Pong"));
    ctx.reply(res).await?;
    Ok(())
}
