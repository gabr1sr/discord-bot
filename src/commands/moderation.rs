use std::time::{Duration, SystemTime};

use crate::models::Punishment;
use crate::utils::user_ids_from;
use crate::{Context, Error};
use serenity::all::{
    ChannelId, EditChannel, GuildId, Http, PermissionOverwrite, PermissionOverwriteType,
    Permissions, RoleId,
};
use serenity::builder::EditMember;
use serenity::model::{channel::GuildChannel, id::UserId};
use sqlx::types::chrono::{DateTime, FixedOffset, Utc};

#[derive(poise::ChoiceParameter, Debug)]
enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
    Days,
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    category = "Moderation"
)]
pub async fn kick(ctx: Context<'_>, users: String, reason: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if users.is_empty() || user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    if let None = check_greater_hierarchy(&ctx, ctx.author().id, &user_ids) {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();

    let (punished_users, not_punished_users) =
        kick_users(ctx, guild_id, &mut user_ids, &reason, None).await?;

    let mut message = String::new();

    if !punished_users.is_empty() {
        let punished_mentions = user_ids_to_mentions(punished_users).join(", ");
        let response = format!(
            ":white_check_mark: **Successfully kicked members:** {}\n",
            punished_mentions
        );
        message.push_str(&response);
    }

    if !not_punished_users.is_empty() {
        let not_punished_mentions = user_ids_to_mentions(not_punished_users).join(", ");
        let response = format!(
            ":warning: **Failed to kick members:** {}\n",
            not_punished_mentions
        );
        message.push_str(&response);
    }

    if !reason.is_empty() {
        let response = format!(":information: **Kick reason:** {}", reason);
        message.push_str(&response);
    }

    if message.is_empty() {
        message.push_str("Failed to execute kick command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    category = "Moderation"
)]
pub async fn timeout(
    ctx: Context<'_>,
    users: String,
    time: i64,
    unit: TimeUnit,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    if let None = check_greater_hierarchy(&ctx, ctx.author().id, &user_ids) {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let duration_i64 = match unit {
        TimeUnit::Seconds => time,
        TimeUnit::Minutes => time * 60,
        TimeUnit::Hours => time * 60 * 60,
        TimeUnit::Days => {
            let time_fix = if time > 28 { 28 } else { time };
            time_fix * 60 * 60 * 24
        }
    };

    let duration = to_iso8601(duration_i64);
    let guild_id = ctx.guild_id().unwrap();
    let (punished_users, not_punished_users) =
        timeout_users(ctx, guild_id, &mut user_ids, duration, None).await?;

    let mut message = String::new();

    if !punished_users.is_empty() {
        let punished_mentions = user_ids_to_mentions(punished_users).join(", ");
        let response = format!(
            ":white_check_mark: **Successfully timed out members:** {}\n",
            punished_mentions
        );
        message.push_str(&response);
    }

    if !not_punished_users.is_empty() {
        let not_punished_mentions = user_ids_to_mentions(not_punished_users).join(", ");
        let response = format!(
            ":warning: **Failed to time out members:** {}\n",
            not_punished_mentions
        );
        message.push_str(&response);
    }

    if time > 0 {
        let units = format!("{unit:?}").to_lowercase();
        let response = format!(":information: **Time out duration:** {} {}", time, units);
        message.push_str(&response);
    }

    if message.is_empty() {
        message.push_str("Failed to execute kick command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    category = "Moderation"
)]
pub async fn untimeout(ctx: Context<'_>, users: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    if let None = check_greater_hierarchy(&ctx, ctx.author().id, &user_ids) {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let (unpunished_users, not_unpunished_users) =
        untimeout_users(ctx, guild_id, &mut user_ids).await?;

    let mut message = String::new();

    if !unpunished_users.is_empty() {
        let mentions = user_ids_to_mentions(unpunished_users).join(", ");
        let res = format!(
            ":white_check_mark: **Successfully untimedout members:** {}\n",
            mentions
        );
        message.push_str(&res);
    }

    if !not_unpunished_users.is_empty() {
        let mentions = user_ids_to_mentions(not_unpunished_users).join(", ");
        let res = format!(":warning: **Failed to untimeout members:** {}\n", mentions);
        message.push_str(&res);
    }

    if message.is_empty() {
        message.push_str("Failed to execute untimeout command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    category = "Moderation"
)]
pub async fn ban(ctx: Context<'_>, users: String, reason: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    if let None = check_greater_hierarchy(&ctx, ctx.author().id, &user_ids) {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let (punished_users, not_punished_users) =
        ban_users(ctx, guild_id, &mut user_ids, &reason, None).await?;

    let mut message = String::new();

    if !punished_users.is_empty() {
        let punished_mentions = user_ids_to_mentions(punished_users).join(", ");
        let response = format!(
            ":white_check_mark: **Successfully banned members:** {}\n",
            punished_mentions
        );
        message.push_str(&response);
    }

    if !not_punished_users.is_empty() {
        let not_punished_mentions = user_ids_to_mentions(not_punished_users).join(", ");
        let response = format!(
            ":warning: **Failed to ban members:** {}\n",
            not_punished_mentions
        );
        message.push_str(&response);
    }

    if !reason.is_empty() {
        let response = format!(":information: **Ban reason:** {}", reason);
        message.push_str(&response);
    }

    if message.is_empty() {
        message.push_str("Failed to execute ban command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "BAN_MEMBERS",
    category = "Moderation"
)]
pub async fn unban(ctx: Context<'_>, users: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();
    let (unpunished_users, not_unpunished_users) =
        unban_users(ctx, guild_id, &mut user_ids).await?;

    let mut message = String::new();

    if !unpunished_users.is_empty() {
        let mentions = user_ids_to_mentions(unpunished_users).join(", ");
        let res = format!(
            ":white_check_mark: **Successfully unbanned members:** {}\n",
            mentions
        );
        message.push_str(&res);
    }

    if !not_unpunished_users.is_empty() {
        let mentions = user_ids_to_mentions(not_unpunished_users).join(", ");
        let res = format!(":warning: **Failed to unban members:** {}\n", mentions);
        message.push_str(&res);
    }

    if message.is_empty() {
        message.push_str("Failed to execute unban command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MODERATE_MEMBERS",
    category = "Moderation"
)]
pub async fn strike(ctx: Context<'_>, users: String, reason: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    if let None = check_greater_hierarchy(&ctx, ctx.author().id, &user_ids) {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let (punished_users, not_punished_users) =
        strike_users(ctx, &mut user_ids, &reason, None).await?;

    let mut message = String::new();

    if !punished_users.is_empty() {
        let punished_mentions = user_ids_to_mentions(punished_users).join(", ");
        let response = format!(
            ":white_check_mark: **Successfully striked members:** {}\n",
            punished_mentions
        );
        message.push_str(&response);
    }

    if !not_punished_users.is_empty() {
        let not_punished_mentions = user_ids_to_mentions(not_punished_users).join(", ");
        let response = format!(
            ":warning: **Failed to strike members:** {}\n",
            not_punished_mentions
        );
        message.push_str(&response);
    }

    if !reason.is_empty() {
        let response = format!(":information: **Strike reason:** {}", reason);
        message.push_str(&response);
    }

    if message.is_empty() {
        message.push_str("Failed to execute strike command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS | BAN_MEMBERS | MODERATE_MEMBERS",
    category = "Moderation"
)]
pub async fn punish(ctx: Context<'_>, id: i32, users: String, reason: String) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let mut user_ids: Vec<UserId> = user_ids_from(&users);

    if user_ids.is_empty() {
        ctx.reply("You must provide at least 1 valid user mention or user ID.")
            .await?;
        return Ok(());
    }

    let guild_id = ctx.guild_id().unwrap();

    let infraction = ctx.data().database.get_infraction(id).await;

    if let Err(_) = infraction {
        ctx.reply("This infraction ID doesn't exists!").await?;
        return Ok(());
    }

    let infraction = infraction.unwrap();

    if let None = check_greater_hierarchy(&ctx, ctx.author().id, &user_ids) {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let (punished_users, not_punished_users) = match infraction.punishment {
        Punishment::Ban => {
            ban_users(ctx, guild_id, &mut user_ids, &reason, Some(infraction.id)).await?
        }
        Punishment::Timeout => {
            timeout_users(
                ctx,
                guild_id,
                &mut user_ids,
                to_iso8601(infraction.duration),
                Some(infraction.id),
            )
            .await?
        }
        Punishment::Strike => {
            strike_users(ctx, &mut user_ids, &reason, Some(infraction.id)).await?
        }
        Punishment::Kick => {
            kick_users(ctx, guild_id, &mut user_ids, &reason, Some(infraction.id)).await?
        }
    };

    let mut message = String::new();

    if !punished_users.is_empty() {
        let punished_mentions = user_ids_to_mentions(punished_users).join(", ");
        let response = format!(
            ":white_check_mark: **Successfully punished members:** {}\n",
            punished_mentions
        );
        message.push_str(&response);
    }

    if !not_punished_users.is_empty() {
        let not_punished_mentions = user_ids_to_mentions(not_punished_users).join(", ");
        let response = format!(
            ":warning: **Failed to punish members:** {}\n",
            not_punished_mentions
        );
        message.push_str(&response);
    }

    let punishment_type = format!("{:?}", infraction.punishment).to_lowercase();
    let response = format!(":information: **Punishment type:** {:?}\n", punishment_type);
    message.push_str(&response);

    if infraction.duration > 0 {
        let duration = format!("{}", infraction.duration).to_lowercase();
        let response = format!(":information: **Punishment duration:** {:?}\n", duration);
        message.push_str(&response);
    }

    if !reason.is_empty() {
        let response = format!(":information: **Punishment reason:** {}", reason);
        message.push_str(&response);
    }

    if message.is_empty() {
        message.push_str("Failed to execute punish command!");
    }

    ctx.reply(message).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    category = "Moderation"
)]
pub async fn slowmode(
    ctx: Context<'_>,
    #[max = 21600] // max = 6 hours
    seconds: u16,
    channel: Option<GuildChannel>,
    #[max = 86400] // max = 24 hours
    duration: Option<u64>,
) -> Result<(), Error> {
    let mut channel = channel.unwrap_or(ctx.guild_channel().await.unwrap());
    let builder = EditChannel::new().rate_limit_per_user(seconds);

    let res = match channel.edit(&ctx, builder).await {
        Ok(_) => format!(":white_check_mark: Slowmode of `{seconds}` seconds enabled with success on channel {channel}!"),
        Err(_) => format!(":x: Failed to enable slowmode on channel {channel}!")
    };

    ctx.reply(res).await?;

    if let Some(duration) = duration {
        tokio::spawn(remove_slowmode_after(channel.id, duration));
    }

    Ok(())
}

async fn remove_slowmode_after(channel_id: ChannelId, duration: u64) -> Result<(), Error> {
    #[derive(serde::Serialize)]
    struct SlowModeChannel {
        rate_limit_per_user: u16,
    }

    tokio::time::sleep(Duration::from_secs(duration)).await;

    let token = std::env::var("DISCORD_TOKEN").unwrap();
    let http = Http::new(&token);
    let map = SlowModeChannel {
        rate_limit_per_user: 0,
    };
    http.edit_channel(channel_id, &map, None).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "MANAGE_CHANNELS",
    required_bot_permissions = "MANAGE_CHANNELS",
    category = "Moderation"
)]
pub async fn lock(ctx: Context<'_>, channel: Option<GuildChannel>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    let channel = channel.unwrap_or(ctx.guild_channel().await.unwrap());

    let permission_types = channel
        .permission_overwrites
        .iter()
        .map(|ov| ov.kind)
        .collect::<Vec<_>>();

    for permission_type in permission_types.into_iter() {
        channel
            .delete_permission(ctx.http(), permission_type)
            .await?;
    }

    let allow = Permissions::empty();

    let deny = Permissions::SEND_MESSAGES
        | Permissions::SEND_TTS_MESSAGES
        | Permissions::ADD_REACTIONS
        | Permissions::SEND_MESSAGES_IN_THREADS
        | Permissions::CREATE_PUBLIC_THREADS
        | Permissions::CREATE_PRIVATE_THREADS;

    let kind = {
        let guild_id = ctx.guild_id().unwrap();
        let role_id = RoleId::new(guild_id.get());
        PermissionOverwriteType::Role(role_id)
    };

    let permissions = PermissionOverwrite { allow, deny, kind };

    let res = match channel.create_permission(ctx.http(), permissions).await {
        Ok(()) => format!(":white_check_mark: Channel {channel} locked with success!"),
        Err(_) => format!(":x: Failed to lock channel {channel}!"),
    };

    ctx.reply(res).await?;
    Ok(())
}

fn to_iso8601(duration: i64) -> String {
    let now = SystemTime::now();
    let datetime_now: DateTime<Utc> = now.into();
    let timestamp_now = datetime_now.timestamp();

    let timestamp = timestamp_now + duration;
    let datetime: DateTime<Utc> = DateTime::from_timestamp(timestamp, 0).unwrap();

    datetime.to_rfc3339()
}

fn from_iso8601(duration: String) -> i64 {
    let now = SystemTime::now();
    let datetime_now: DateTime<Utc> = now.into();
    let timestamp_now = datetime_now.timestamp();

    let datetime: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(duration.as_str()).unwrap();

    datetime.timestamp() - timestamp_now
}

async fn kick_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
    reason: &str,
    infraction: Option<i32>,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut kicked = vec![];
    let mut not_kicked = vec![];

    for user_id in user_ids.iter() {
        match guild_id.kick_with_reason(&ctx, user_id, reason).await {
            Ok(_) => {
                kicked.push(*user_id);
                match infraction {
                    Some(id) => {
                        ctx.data()
                            .database
                            .log_user_infraction(&user_id, id)
                            .await?;
                    }
                    None => {
                        ctx.data()
                            .database
                            .log_user_punishment(&user_id, Punishment::Kick, 0)
                            .await?;
                    }
                };
            }
            Err(_) => not_kicked.push(*user_id),
        };
    }

    Ok((kicked, not_kicked))
}

async fn ban_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
    reason: &str,
    infraction: Option<i32>,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut banned = vec![];
    let mut not_banned = vec![];

    for user_id in user_ids.iter() {
        match guild_id.ban_with_reason(&ctx, user_id, 0, reason).await {
            Ok(_) => {
                banned.push(*user_id);
                match infraction {
                    Some(id) => {
                        ctx.data()
                            .database
                            .log_user_infraction(&user_id, id)
                            .await?;
                    }
                    None => {
                        ctx.data()
                            .database
                            .log_user_punishment(&user_id, Punishment::Ban, 0)
                            .await?;
                    }
                };
            }
            Err(_) => not_banned.push(*user_id),
        };
    }

    Ok((banned, not_banned))
}

async fn unban_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut unbanned = Vec::new();
    let mut not_unbanned = Vec::new();

    for user_id in user_ids.iter() {
        match guild_id.unban(&ctx, user_id).await {
            Ok(_) => unbanned.push(*user_id),
            Err(_) => not_unbanned.push(*user_id),
        };
    }

    Ok((unbanned, not_unbanned))
}

async fn untimeout_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut untimedout = Vec::new();
    let mut not_untimedout = Vec::new();

    for user_id in user_ids.iter() {
        let builder = EditMember::new().enable_communication();

        match guild_id.edit_member(&ctx, *user_id, builder).await {
            Ok(_) => untimedout.push(*user_id),
            Err(_) => not_untimedout.push(*user_id),
        };
    }

    Ok((untimedout, not_untimedout))
}

async fn timeout_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
    duration: String,
    infraction: Option<i32>,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut timedout = vec![];
    let mut not_timedout = vec![];
    let duration_i64 = from_iso8601(duration.clone());

    for user_id in user_ids.iter() {
        let builder = EditMember::new().disable_communication_until(duration.clone());

        match guild_id.edit_member(&ctx, *user_id, builder).await {
            Ok(_) => {
                timedout.push(*user_id);
                match infraction {
                    Some(id) => {
                        ctx.data()
                            .database
                            .log_user_infraction(&user_id, id)
                            .await?;
                    }
                    None => {
                        ctx.data()
                            .database
                            .log_user_punishment(&user_id, Punishment::Timeout, duration_i64)
                            .await?;
                    }
                };
            }
            Err(_) => not_timedout.push(*user_id),
        };
    }

    Ok((timedout, not_timedout))
}

async fn strike_users(
    ctx: Context<'_>,
    user_ids: &mut Vec<UserId>,
    reason: &str,
    infraction: Option<i32>,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut striked = vec![];

    for user_id in user_ids.iter() {
        match user_id.create_dm_channel(&ctx).await {
            Ok(channel) => {
                let res = format!("You received a strike:\n{}", reason);
                match channel.say(&ctx, res).await {
                    Ok(_) => (),
                    Err(_) => (),
                }
            }
            Err(_) => (),
        };

        match infraction {
            Some(id) => {
                ctx.data()
                    .database
                    .log_user_infraction(&user_id, id)
                    .await?;
            }
            None => {
                ctx.data()
                    .database
                    .log_user_punishment(&user_id, Punishment::Strike, 0)
                    .await?;
            }
        };

        striked.push(*user_id);
    }

    Ok((striked, vec![]))
}

fn check_greater_hierarchy(ctx: &Context<'_>, caller: UserId, users: &[UserId]) -> Option<UserId> {
    let guild = ctx.guild().unwrap();

    for user_id in users.iter() {
        let result = match guild.greater_member_hierarchy(&ctx, caller, user_id) {
            Some(id) => caller == id,
            None => false,
        };

        if !result {
            return None;
        }
    }

    Some(caller)
}

fn user_ids_to_mentions(user_ids: Vec<UserId>) -> Vec<String> {
    user_ids
        .iter()
        .map(|u| u.get())
        .map(|id| format!("<@{id}>"))
        .collect()
}
