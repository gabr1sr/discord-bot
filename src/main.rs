use std::sync::Arc;
use std::time::Duration;

use dotenv::dotenv;
use poise::serenity_prelude as serenity;

pub mod commands;
pub mod database;
pub mod models;
pub mod translation;
pub mod utils;

use database::Database;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

pub struct Data {
    translations: translation::Translations,
    database: Database,
    bang_channel: Arc<Mutex<u64>>,
    bang_available: Arc<Mutex<bool>>,
    bang_handles: Arc<Mutex<Vec<JoinHandle<Result<(), Error>>>>>,
}

async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => panic!("Failed to start bot: {:?}", error),
        poise::FrameworkError::Command { error, ctx, .. } => {
            println!("Error in command `{}`: {:?}", ctx.command().name, error);
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
    let db_url = std::env::var("DATABASE_URL").expect("missing DATABASE_URL");

    let database = Database::new(db_url).await.unwrap();

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let mut commands = vec![
        commands::misc::ping(),
        commands::utility::help(),
        commands::misc::database(),
        commands::infractions::infractions(),
        commands::moderation::punish(),
        commands::moderation::kick(),
        commands::moderation::timeout(),
        commands::moderation::ban(),
        commands::moderation::unban(),
        commands::moderation::untimeout(),
        commands::moderation::strike(),
        commands::tags::tag(),
        commands::emoji::emoji(),
        commands::bang::startbang(),
        commands::bang::bang(),
        commands::bang::stopbang(),
    ];

    let translations = translation::read_ftl().expect("failed to read translation files");
    translation::apply_translations(&translations, &mut commands);

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands,
            prefix_options: poise::PrefixFrameworkOptions {
                prefix: Some("ko!".into()),
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
                Ok(Data {
                    translations,
                    database,
                    bang_channel: Arc::new(Mutex::new(0)),
                    bang_available: Arc::new(Mutex::new(false)),
                    bang_handles: Arc::new(Mutex::new(Vec::new())),
                })
            })
        })
        .build();

    let client = serenity::ClientBuilder::new(token, intents)
        .framework(framework)
        .await;

    client.unwrap().start().await.unwrap();
}
