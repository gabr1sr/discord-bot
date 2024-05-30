use crate::database::Database;
use crate::models::{AnimalModel, BangPointModel};
use crate::{Context, Error};
use rand::seq::SliceRandom;
use rand::Rng;
use serenity::all::{ChannelId, Http};
use serenity::json::json;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

#[poise::command(slash_command, prefix_command, category = "Bang")]
pub async fn startbang(ctx: Context<'_>, channel: ChannelId) -> Result<(), Error> {
    ctx.defer().await?;
    let mut bang_channel = ctx.data().bang_channel.lock().await;
    *bang_channel = channel.get();
    drop(bang_channel);

    let mut handles = ctx.data().bang_handles.lock().await;
    let bang_available = Arc::clone(&ctx.data().bang_available);
    let database = Arc::clone(&ctx.data().database);
    let last_animal = Arc::clone(&ctx.data().last_animal);
    handles.push(tokio::spawn(generate_bang(
        channel,
        bang_available,
        database,
        last_animal,
    )));

    ctx.reply(format!("Bang mini-game started at channel: <#{channel}>"))
        .await?;
    Ok(())
}

async fn generate_bang(
    channel_id: ChannelId,
    bang_available: Arc<Mutex<bool>>,
    database: Arc<Database>,
    last_animal: Arc<Mutex<AnimalModel>>,
) -> Result<(), Error> {
    let interval: u64 = {
        let min_interval = 3; // 5 minute
        let max_interval = 6; // 10 minutes
        rand::thread_rng().gen_range(min_interval..max_interval)
    };

    tokio::time::sleep(Duration::from_secs(interval)).await;

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let http = Http::new(&token);

    let animals = database.get_animals().await.unwrap();

    let animal = animals.choose(&mut rand::thread_rng()).unwrap();

    let map = json!({ "content": format!("{} A wild {} appeared!", animal.emoji.clone(), animal.animal.clone()) });
    http.send_message(channel_id, vec![], &map).await?;

    let mut last_animal_mut = last_animal.lock().await;
    *last_animal_mut = AnimalModel {
        id: animal.id,
        emoji: animal.emoji.clone(),
        animal: animal.animal.clone(),
        points: animal.points,
    };

    let mut is_bang_available = bang_available.lock().await;
    *is_bang_available = true;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Bang")]
pub async fn bang(ctx: Context<'_>) -> Result<(), Error> {
    let mut bang_available = ctx.data().bang_available.lock().await;

    let res = if *bang_available {
        *bang_available = false;
        drop(bang_available);

        let mut handles = ctx.data().bang_handles.lock().await;
        let channel_id = ctx.channel_id();
        let arc_bang_available = Arc::clone(&ctx.data().bang_available);
        let database = Arc::clone(&ctx.data().database);
        let arc_last_animal = Arc::clone(&ctx.data().last_animal);

        let last_animal = arc_last_animal.lock().await;
        let animal_emoji = last_animal.emoji.clone();
        let animal_name = last_animal.animal.clone();
        let animal_points = last_animal.points;

        drop(last_animal);

        let user_id = ctx.author().id.to_string();

        if let Ok(_) = database
            .create_or_add_user_bang_points(user_id, animal_points)
            .await
        {
            handles.push(tokio::spawn(generate_bang(
                channel_id,
                arc_bang_available,
                database,
                arc_last_animal,
            )));

            format!(
                "Nice! You just shot a {} {} and gained `{}` points!",
                animal_emoji, animal_name, animal_points
            )
        } else {
            format!("Failed to update user points! Stopping bang minigame...")
        }
    } else {
        format!("Bang isn't available yet!")
    };

    ctx.reply(res).await?;

    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Bang")]
pub async fn stopbang(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;
    let mut handles = ctx.data().bang_handles.lock().await;

    // TODO: find a more efficient way to manage handles

    while let Some(handle) = handles.pop() {
        if handle.is_finished() {
            continue;
        }

        handle.abort();
    }

    let mut bang_available = ctx.data().bang_available.lock().await;
    *bang_available = false;

    ctx.reply("Bang mini-game stopped!").await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command, category = "Bang")]
pub async fn ranking(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer().await?;

    if let Ok(result) = ctx.data().database.get_bang_ranking().await {
        let res = parse_ranking(&result);
        ctx.reply(res).await?;
        return Ok(());
    }

    ctx.reply("No one is ranked!").await?;
    Ok(())
}

fn parse_ranking(bang_points: &[BangPointModel]) -> String {
    if bang_points.is_empty() {
        return "No one is ranked!".to_owned();
    }

    let mut lines = Vec::new();
    lines.extend(
        bang_points
            .iter()
            .map(|b| format!("- `{}` | `{}` points", b.user_id, b.points)),
    );
    lines.join("\n")
}
