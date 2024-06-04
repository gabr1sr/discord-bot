use std::time::SystemTime;

use crate::models::{InfractionModel, Punishment, Severity};
use crate::utils::user_ids_from;
use crate::{Context, Error};
use serenity::all::GuildId;
use serenity::builder::EditMember;
use serenity::model::id::UserId;
use sqlx::types::chrono::{DateTime, FixedOffset, NaiveDateTime, Utc};

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

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
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

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
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

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
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

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
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

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
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
pub async fn punish(
    ctx: Context<'_>,
    id: i32,
    users: String,
    message: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let users_str = users.as_str();
    let mut user_ids: Vec<UserId> = user_ids_from(users_str);

    let guild_id = ctx.guild_id().unwrap();

    let result = sqlx::query_as!(
        InfractionModel,
        r#"SELECT id, severity AS "severity!: Severity", punishment AS "punishment!: Punishment", duration FROM infractions WHERE id = $1"#,
        id
    )
        .fetch_one(&ctx.data().database.pool)
        .await;

    if let Err(_) = result {
        ctx.reply("This infraction ID doesn't exists!").await?;
        return Ok(());
    }

    let infraction = result.unwrap();

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let result = match infraction.punishment {
        Punishment::Ban => {
            ban_users(ctx, guild_id, &mut user_ids, &message, Some(infraction.id)).await?
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
            strike_users(ctx, &mut user_ids, &message, Some(infraction.id)).await?
        }
        Punishment::Kick => {
            kick_users(ctx, guild_id, &mut user_ids, &message, Some(infraction.id)).await?
        }
    };

    let res = punish_response(result);
    ctx.reply(res).await?;
    Ok(())
}

fn to_iso8601(duration: i64) -> String {
    let now = SystemTime::now();
    let datetime_now: DateTime<Utc> = now.into();
    let timestamp_now = datetime_now.timestamp();

    let timestamp = timestamp_now + duration;
    let naive_datetime: NaiveDateTime = NaiveDateTime::from_timestamp_opt(timestamp, 0).unwrap();
    let datetime: DateTime<Utc> = DateTime::from_naive_utc_and_offset(naive_datetime, Utc);

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

async fn assert_highest_role(ctx: &Context<'_>, user_ids: &mut Vec<UserId>) -> Result<bool, Error> {
    let author_member = ctx.author_member().await.unwrap();
    let (_, author_role_position) = author_member.highest_role_info(ctx).unwrap();

    let guild_id = ctx.guild_id().unwrap();

    for user_id in user_ids.iter() {
        let member = guild_id.member(&ctx, user_id).await.unwrap();

        return Ok(match member.highest_role_info(&ctx) {
            Some((_, member_role_position)) => author_role_position >= member_role_position,
            None => true,
        });
    }

    Ok(true)
}

fn punish_response((punished_users, not_punished_users): (Vec<UserId>, Vec<UserId>)) -> String {
    let punished_mentions = user_ids_to_mentions(punished_users);
    let not_punished_mentions = user_ids_to_mentions(not_punished_users);

    format!(
        "Punished users: {}\nNot punished users: {}",
        punished_mentions.join(", "),
        not_punished_mentions.join(", ")
    )
}

fn user_ids_to_mentions(user_ids: Vec<UserId>) -> Vec<String> {
    user_ids
        .iter()
        .map(|u| u.get())
        .map(|id| format!("<@{id}>"))
        .collect()
}
