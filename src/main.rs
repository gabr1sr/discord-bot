use poise::serenity_prelude as serenity;
use dotenv::dotenv;

struct Data {}
type Error = Box<dyn std::error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;

#[poise::command(slash_command, prefix_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let res = format!("{}'s account was created at {}", u.name, u.created_at());
    ctx.say(res).await?;
    Ok(())
}

#[poise::command(slash_command, prefix_command)]
async fn invidious(
    ctx: Context<'_>,
    #[description = "YouTube URL"] url: Option<String>,
) -> Result<(), Error> {
    let video_url = url.unwrap_or_else(|| "https://www.youtube.com/".to_string());
    let new_url = video_url.replace("www.youtube.com", "invidious.nerdvpn.de");
    ctx.say(new_url).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    let token = std::env::var("DISCORD_TOKEN").expect("missing DISCORD_TOKEN");
    let intents = serenity::GatewayIntents::non_privileged();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![age(), invidious()],
            ..Default::default()
        })
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
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
