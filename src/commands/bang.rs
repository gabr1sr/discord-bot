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
    handles.push(tokio::spawn(generate_bang(channel, bang_available)));

    ctx.reply(format!("Bang mini-game started at channel: <#{channel}>"))
        .await?;
    Ok(())
}

async fn generate_bang(
    channel_id: ChannelId,
    bang_available: Arc<Mutex<bool>>,
) -> Result<(), Error> {
    let interval: u64 = {
        let min_interval = 300; // 5 minute
        let max_interval = 1200; // 20 minutes
        rand::thread_rng().gen_range(min_interval..max_interval)
    };
    
    tokio::time::sleep(Duration::from_secs(interval)).await;

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let http = Http::new(&token);

    let animals = vec![
        ":duck: A wild duck appeared!",
        ":boar: A wild boar appeared!",
        ":deer: A wild deer appeared!",
        ":rabbit: A wild rabbit appeared!",
    ];

    let animal = animals.choose(&mut rand::thread_rng());

    let map = json!({ "content": animal });
    http.send_message(channel_id, vec![], &map).await?;

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
        handles.push(tokio::spawn(generate_bang(channel_id, arc_bang_available)));

        format!("Nice shot!")
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
