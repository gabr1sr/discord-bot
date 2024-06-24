use crate::{Context, Error};
use serenity::all::EmojiParseError;

pub async fn handle_emoji_parse_error(
    error: Box<EmojiParseError>,
    input: Option<String>,
    _: Context<'_>,
) -> Result<String, Error> {
    Ok(match *error {
        EmojiParseError::NotFoundOrMalformed => format!(
            ":x: Failed to parse emoji: `{:?}`",
            input.unwrap_or(error.to_string())
        ),
        error => dbg!(error).to_string(),
    })
}
