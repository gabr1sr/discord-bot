use crate::{utils::user_ids_from, Context, Error};
use poise::{
    serenity_prelude::{GetMessages, GuildChannel},
    CreateReply,
};
use serenity::{
    all::{CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, GuildId, Http, Timestamp, UserId},
    futures::future::join_all,
};

#[poise::command(slash_command, prefix_command)]
pub async fn experimental(ctx: Context<'_>, users: String) -> Result<(), Error> {
    let users = user_ids_from(&users);
    ctx.reply(format!("```rust\n{:?}\n```", dbg!(users)))
        .await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "MANAGE_MESSAGES",
    required_permissions = "MANAGE_MESSAGES",
    category = "Moderation",
    guild_only
)]
pub async fn clear(
    ctx: Context<'_>,
    #[max = 100]
    #[min = 100]
    amount: u8,
    #[rest] channel: Option<GuildChannel>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let channel = channel.unwrap_or(ctx.guild_channel().await.unwrap());
    let builder = GetMessages::new().limit(amount);

    let Ok(messages) = channel.messages(&ctx, builder).await else {
        ctx.reply(":x: Failed to retrieve messages from the channel {channel}!")
            .await?;

        return Ok(());
    };

    if messages.is_empty() {
        ctx.reply(":warning: The channel {channel} has no messages!")
            .await?;

        return Ok(());
    };

    let count = messages.len();

    let Ok(()) = channel.delete_messages(&ctx.http(), messages).await else {
        ctx.reply(":x: No messages deleted!").await?;
        return Ok(());
    };

    let res =
        format!(":white_check_mark: `{count}` messages were successfully deleted from the channel {channel}!");

    let reply = CreateReply::default().content(res).ephemeral(true);
    ctx.send(reply).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "MANAGE_MESSAGES",
    required_permissions = "MANAGE_MESSAGES",
    category = "Moderation",
    guild_only
)]
pub async fn kick(ctx: Context<'_>, users: String, reason: Option<String>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let reason = reason.as_deref();
    let users = user_ids_from(&users);
    let users = user_ids_below_user(&ctx, users, ctx.author().id);
    let guild_id = ctx.guild_id().unwrap();
    let http = ctx.http();
    let results = kick_members(http, guild_id, users, reason).await?;

    let punished_users = results
        .into_iter()
        .filter_map(|r| r.ok())
        .collect::<Vec<UserId>>();

    let embed = build_kick_embed(&ctx, punished_users, reason.unwrap_or("No reason provided"));
    let builder = CreateReply::default().embed(embed).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

fn build_kick_embed(ctx: &Context<'_>, user_ids: Vec<UserId>, reason: &str) -> CreateEmbed {
    let author = ctx.author();
    let client = ctx.cache().current_user();
    let author_avatar_url = author.avatar_url().unwrap_or(author.default_avatar_url());
    let client_avatar_url = client.avatar_url().unwrap_or(client.default_avatar_url());
    let embed_author = CreateEmbedAuthor::new(&author.name).icon_url(author_avatar_url);
    let embed_footer = CreateEmbedFooter::new(&client.name).icon_url(client_avatar_url);
    let timestamp = Timestamp::now();
    let amount = user_ids.len();
    let description = format!("**{amount}** users kicked out!");

    CreateEmbed::new()
        .author(embed_author)
        .footer(embed_footer)
        .timestamp(timestamp)
        .color(0x3BA55C)
        .title("Kick")
        .description(description)
        .field("Reason", reason, false)
        .field("Users", format_user_ids_list(user_ids), false)
}

fn format_user_ids_list(user_ids: Vec<UserId>) -> String {
    user_ids
        .into_iter()
        .map(|u| format!("- <@{}>", u.get()))
        .collect::<Vec<String>>()
        .join("\n")
}

fn user_ids_below_user(ctx: &Context<'_>, user_ids: Vec<UserId>, user_id: UserId) -> Vec<UserId> {
    let guild = ctx.guild().unwrap();

    user_ids
        .into_iter()
        .filter(|u| guild.greater_member_hierarchy(&ctx, user_id, u).is_some())
        .collect()
}

async fn kick_members(
    http: impl AsRef<Http>,
    guild_id: GuildId,
    user_ids: impl IntoIterator<Item = UserId>,
    reason: Option<&str>,
) -> Result<Vec<Result<UserId, Error>>, Error> {
    let futures = user_ids
        .into_iter()
        .map(|user_id| kick_member(&http, guild_id, user_id, reason))
        .collect::<Vec<_>>();

    Ok(join_all(futures).await)
}

async fn kick_member(
    http: impl AsRef<Http>,
    guild_id: GuildId,
    user_id: UserId,
    reason: Option<&str>,
) -> Result<UserId, Error> {
    http.as_ref().kick_member(guild_id, user_id, reason).await?;
    Ok(user_id)
}
