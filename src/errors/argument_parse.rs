use crate::{Context, Error};
use serenity::all::EmojiParseError;

pub async fn handle_emoji_parse_error(
    error: Box<EmojiParseError>,
    input: Option<String>,
    _: Context<'_>,
) -> Result<String, Error> {
    Ok(match *error {
        EmojiParseError::NotFoundOrMalformed => format!(
            ":x: Emoji not found or malformed: `{}`",
            input.unwrap_or(error.to_string())
        ),
        error => dbg!(error).to_string(),
    })
}
