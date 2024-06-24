use ::serenity::all::EmojiParseError;
use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use std::time::Duration;

pub mod commands;
pub mod utils;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub struct Data {}

async fn argument_parse_error_handler(
    error: Error,
    input: Option<String>,
    ctx: Context<'_>,
) -> Result<(), serenity::Error> {
    ctx.defer_ephemeral().await?;

    if let Ok(error) = error.downcast::<EmojiParseError>() {
        let res = match *error {
            EmojiParseError::NotFoundOrMalformed => format!(
                "Failed to parse invalid emoji: `{}`\nPlease provide a valid emoji.",
                input.unwrap()
            ),
            _ => error.to_string(),
        };

        ctx.reply(res).await?;
    }

    Ok(())
}

// TODO: create own error handler
// builtin on_error:
// https://docs.rs/poise/latest/src/poise/builtins/mod.rs.html#35-196
async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
        }
        poise::FrameworkError::ArgumentParse {
            error, input, ctx, ..
        } => {
            if let Err(e) = argument_parse_error_handler(error, input, ctx).await {
                println!("Error while handling error: {}", e);
            }
        }
        error => {
            if let Err(e) = poise::builtins::on_error(error).await {
                println!("Error while handling error: {}", e)
            }
        }
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let commands = vec![commands::emojis::emoji()];

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("k!".into()),
                case_insensitive_commands: true,
                edit_tracker: Some(Arc::new(poise::EditTracker::for_timespan(
                    Duration::from_secs(3600),
                ))),
                ..Default::default()
            },
            on_error: |error| Box::pin(on_error(error)),
            pre_command: |ctx| {
                Box::pin(async move {
                    println!("Executing command {}...", ctx.command().qualified_name);
                })
            },
            post_command: |ctx| {
                Box::pin(async move {
                    println!("Executed command {}!", ctx.command().qualified_name);
                })
            },
            command_check: Some(|ctx| Box::pin(async move { Ok(!ctx.author().bot) })),
            skip_checks_for_owners: false,
            event_handler: |_ctx, event, _framework, _data| {
                Box::pin(async move {
                    println!(
                        "Got an event in event handler: {:?}",
                        event.snake_case_name()
                    );
                    Ok(())
                })
            },
            ..Default::default()
        })
        .setup(|ctx, ready, framework| {
            Box::pin(async move {
                println!("Logged in as {}", ready.user.name);
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {})
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
