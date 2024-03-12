use std::time::SystemTime;

use crate::models::{InfractionModel, Punishment, Severity, UserInfractionModel};
use crate::utils::user_ids_from;
use crate::{Context, Error};
use serenity::all::GuildId;
use serenity::builder::EditMember;
use serenity::model::id::UserId;
use sqlx::types::chrono::{DateTime, NaiveDateTime, Utc};

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS",
    category = "Moderation"
)]
pub async fn kick(
    ctx: Context<'_>,
    users: String,
    reason: String,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let users_str = users.as_str();
    let mut user_ids: Vec<UserId> = user_ids_from(users_str);
    let guild_id = ctx.guild_id().unwrap();

    if !assert_highest_role(&ctx, &mut user_ids).await.unwrap() {
        ctx.reply("One of the users have a role higher than yours.")
            .await?;
        return Ok(());
    }

    let result = kick_users(ctx, guild_id, &mut user_ids, reason).await?;
    let res = punish_response(result);
    ctx.reply(res).await?;
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
        Punishment::Ban => ban_users(ctx, guild_id, &mut user_ids, message, infraction.id).await?,
        Punishment::Timeout => {
            timeout_users(
                ctx,
                guild_id,
                &mut user_ids,
                to_iso8601(infraction.duration),
                infraction.id,
            )
            .await?
        }
        Punishment::Strike => strike_users(ctx, &mut user_ids, message, infraction.id).await?,
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

async fn kick_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
    reason: String,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut kicked = Vec::new();
    let mut not_kicked = Vec::new();

    for user_id in user_ids.iter() {
        match guild_id
            .kick_with_reason(&ctx, user_id, reason.as_str())
            .await {
            Ok(_) => kicked.push(*user_id),
            Err(_) => not_kicked.push(*user_id),
        };
    }

    Ok((kicked, not_kicked))
}

async fn ban_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
    message: String,
    infraction_id: i32,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut banned = Vec::new();
    let mut not_banned = Vec::new();

    for user_id in user_ids.iter() {
        match guild_id
            .ban_with_reason(ctx, user_id, 0, message.as_str())
            .await
        {
            Ok(_) => banned.push(*user_id),
            Err(_) => not_banned.push(*user_id),
        };

        log_punishment(&ctx, user_id, infraction_id).await?;
    }

    Ok((banned, not_banned))
}

async fn timeout_users(
    ctx: Context<'_>,
    guild_id: GuildId,
    user_ids: &mut Vec<UserId>,
    duration: String,
    infraction_id: i32,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut timedout = Vec::new();
    let mut not_timedout = Vec::new();

    for user_id in user_ids.iter() {
        let builder = EditMember::new().disable_communication_until(duration.clone());

        match guild_id.edit_member(ctx, *user_id, builder).await {
            Ok(_) => timedout.push(*user_id),
            Err(_) => not_timedout.push(*user_id),
        };

        log_punishment(&ctx, user_id, infraction_id).await?;
    }

    Ok((timedout, not_timedout))
}

async fn strike_users(
    ctx: Context<'_>,
    user_ids: &mut Vec<UserId>,
    message: String,
    infraction_id: i32,
) -> Result<(Vec<UserId>, Vec<UserId>), Error> {
    let mut striked: Vec<UserId> = Vec::new();

    for user_id in user_ids.iter() {
        let channel = user_id.create_dm_channel(ctx).await.unwrap();
        let res = format!("You received a strike:\n{}", message.clone());
        channel.say(ctx, res).await.unwrap();
        striked.push(*user_id);
        log_punishment(&ctx, user_id, infraction_id).await?;
    }

    Ok((striked, Vec::new()))
}

async fn assert_highest_role(ctx: &Context<'_>, user_ids: &mut Vec<UserId>) -> Result<bool, Error> {
    let author_member = ctx.author_member().await.unwrap();
    let (_, author_role_position) = author_member.highest_role_info(ctx).unwrap();

    let guild_id = ctx.guild_id().unwrap();

    for user_id in user_ids.iter() {
        let member = guild_id.member(ctx, user_id).await.unwrap();
        let (_, member_role_position) = member.highest_role_info(ctx).unwrap();

        if member_role_position >= author_role_position {
            return Ok(false);
        }
    }

    Ok(true)
}

async fn log_punishment(
    ctx: &Context<'_>,
    user_id: &UserId,
    infraction_id: i32,
) -> Result<(), Error> {
    let user_infraction = sqlx::query_as!(
        UserInfractionModel,
        r#"INSERT INTO user_infractions (user_id, infraction_id) VALUES ($1, $2) RETURNING id, user_id, infraction_id, created_at"#,
        user_id.get().to_string(),
        infraction_id,
    )
        .fetch_one(&ctx.data().database.pool)
        .await
        .unwrap();

    println!("{:?}", user_infraction);

    Ok(())
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
    let raw_ids = user_ids.iter().map(|u| u.get());
    let mut mentions = Vec::new();

    for id in raw_ids {
        mentions.push(format!("<@{id}>"));
    }

    mentions
}
