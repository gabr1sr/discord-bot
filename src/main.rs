use dotenv::dotenv;
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use std::time::Duration;

pub mod builders;
pub mod commands;
pub mod errors;
pub mod models;
pub mod utils;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug)]
pub struct Data {
    pool: sqlx::Pool<sqlx::Postgres>,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let database_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let pool = sqlx::postgres::PgPool::connect(&database_url)
        .await
        .expect("failed to connect to database");

    let commands = vec![
        commands::emojis::emoji(),
        commands::emojis::retrieve_emoji_context(),
        commands::emojis::clone_emoji_context(),
        commands::stickers::sticker(),
        commands::stickers::retrieve_sticker_context(),
        commands::stickers::clone_sticker_context(),
        commands::tags::tag(),
        commands::moderation::clear(),
        commands::moderation::kick(),
        commands::moderation::experimental(),
    ];

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
            on_error: |error| Box::pin(errors::on_error(error)),
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
                Ok(Data { pool })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
