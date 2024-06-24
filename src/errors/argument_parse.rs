use crate::{Context, Error};
use poise::CreateReply;
use serenity::all::EmojiParseError;

pub async fn handle_emoji_parse_error(
    error: Box<EmojiParseError>,
    input: Option<String>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    let res = match *error {
        EmojiParseError::NotFoundOrMalformed => format!(
            ":x: Failed to parse emoji: `{:?}`",
            input.unwrap_or(error.to_string())
        ),
        error => dbg!(error).to_string(),
    };

    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}
