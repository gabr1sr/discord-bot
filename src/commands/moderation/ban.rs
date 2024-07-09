use poise::CreateReply;
use serde::{Deserialize, Serialize};
use serenity::{
    all::{
        CreateEmbed, CreateEmbedAuthor, CreateEmbedFooter, GuildId, Http, LightMethod, Request,
        Route, Timestamp, UserId,
    },
    json::to_vec,
};

use crate::{
    utils::{format_user_ids_list, reason_into_header, user_ids_below_user, user_ids_from},
    Context, Error,
};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
struct BulkBanResponse {
    /// The users that were successfully banned.
    pub banned_users: Vec<UserId>,
    /// The users that were not successfully banned.
    pub failed_users: Vec<UserId>,
}

#[poise::command(
    slash_command,
    prefix_command,
    required_bot_permissions = "BAN_MEMBERS",
    required_permissions = "BAN_MEMBERS",
    category = "Moderation"
)]
pub async fn ban(
    ctx: Context<'_>,
    users: String,
    delete_message_seconds: Option<u32>,
    reason: Option<String>,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let reason = reason.as_deref();
    let delete_message_seconds = delete_message_seconds.unwrap_or_default();
    let users = user_ids_from(&users);
    let users = user_ids_below_user(&ctx, users, ctx.author().id);
    let guild_id = ctx.guild_id().unwrap();
    let http = ctx.http();

    let Ok(res) = bulk_ban(guild_id, http, users, delete_message_seconds, reason).await else {
        ctx.reply(format!(":x: Failed to ban users!")).await?;
        return Ok(());
    };

    let embed = build_ban_embed(
        &ctx,
        res.banned_users,
        res.failed_users,
        reason.unwrap_or("No reason provided"),
    );
    let builder = CreateReply::default().embed(embed).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

fn build_ban_embed(
    ctx: &Context<'_>,
    banned_users: Vec<UserId>,
    failed_users: Vec<UserId>,
    reason: &str,
) -> CreateEmbed {
    let author = ctx.author();
    let client = ctx.cache().current_user();
    let author_avatar_url = author.avatar_url().unwrap_or(author.default_avatar_url());
    let client_avatar_url = client.avatar_url().unwrap_or(client.default_avatar_url());
    let embed_author = CreateEmbedAuthor::new(&author.name).icon_url(author_avatar_url);
    let embed_footer = CreateEmbedFooter::new(&client.name).icon_url(client_avatar_url);
    let timestamp = Timestamp::now();
    let banned_amount = banned_users.len();
    let description = format!("**{banned_amount}** users were banned!");

    let embed = CreateEmbed::new()
        .author(embed_author)
        .footer(embed_footer)
        .timestamp(timestamp)
        .title("Ban")
        .description(description)
        .field("Reason", reason, false)
        .color(0x5865F2);

    let embed = match banned_users.is_empty() {
        false => embed.field("Banned Users", format_user_ids_list(banned_users), false),
        true => embed,
    };

    let embed = match failed_users.is_empty() {
        false => embed.field("Failed Users", format_user_ids_list(failed_users), false),
        true => embed,
    };

    embed
}

async fn bulk_ban(
    guild_id: GuildId,
    http: impl AsRef<Http>,
    user_ids: Vec<UserId>,
    delete_message_seconds: u32,
    reason: Option<&str>,
) -> Result<BulkBanResponse, serenity::Error> {
    #[derive(serde::Serialize)]
    struct BulkBan<I> {
        user_ids: I,
        delete_message_seconds: u32,
    }

    let map: BulkBan<Vec<UserId>> = BulkBan {
        user_ids,
        delete_message_seconds,
    };

    bulk_ban_users(http, guild_id, &map, reason).await
}

async fn bulk_ban_users(
    http: impl AsRef<Http>,
    guild_id: GuildId,
    map: &impl serde::Serialize,
    reason: Option<&str>,
) -> Result<BulkBanResponse, serenity::Error> {
    let route = Route::GuildBulkBan { guild_id };
    let body = Some(to_vec(map)?);
    let req = Request::new(route, LightMethod::Post).body(body);

    let req = match reason.map(reason_into_header) {
        Some(map) => req.headers(Some((&map).try_into().unwrap())),
        None => req,
    };

    http.as_ref().fire(req).await
}
