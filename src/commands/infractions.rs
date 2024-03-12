use crate::models::{InfractionModel, Punishment, Severity, UserInfractionModel};
use crate::{Context, Error};
use serenity::model::id::UserId;
use sqlx::postgres::PgQueryResult;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "list", "remove", "user"),
    subcommand_required,
    required_permissions = "ADMINISTRATOR",
    category = "Infractions"
)]
pub async fn infractions(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn add(
    ctx: Context<'_>,
    id: i32,
    severity: Severity,
    punishment: Punishment,
    duration: i64,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    if let Ok(_) = sqlx::query_as!(
        InfractionModel,
        r#"SELECT id, severity AS "severity!: Severity", punishment AS "punishment!: Punishment", duration FROM infractions WHERE id = $1"#,
        id
    )
        .fetch_one(&ctx.data().database.pool)
        .await {
            let res = format!("Infraction with ID `{id}` already exists!");
            ctx.reply(res).await?;
            return Ok(());
        }

    let infraction = sqlx::query_as!(
        InfractionModel,
        r#"INSERT INTO infractions (id, severity, punishment, duration) VALUES ($1, $2, $3, $4) RETURNING id, severity AS "severity!: Severity", punishment AS "punishment!: Punishment", duration"#,
        id,
        severity as Severity,
        punishment as Punishment,
        duration
    )
        .fetch_one(&ctx.data().database.pool)
        .await
        .unwrap();

    let inf = format_infraction(infraction);
    let res = format!("Infraction created!\n{}", inf);
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let result = sqlx::query_as!(
        InfractionModel,
        r#"SELECT id, severity AS "severity!: Severity", punishment AS "punishment!: Punishment", duration FROM infractions ORDER BY id"#,
    )
        .fetch_all(&ctx.data().database.pool)
        .await;

    if let Err(_) = result {
        ctx.reply("No infractions found in the table!").await?;
        return Ok(());
    }

    let infractions = result.unwrap();
    let mut infractions_str = String::new();

    for infraction in infractions {
        let formatted = format_infraction(infraction);
        infractions_str.push_str(formatted.as_str());
    }

    let vec_pages: Vec<&str> = infractions_str.split("\r\n").collect();
    let pages: &[&str] = vec_pages.as_slice();
    poise::samples::paginate(ctx, pages).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, id: i32) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let result: PgQueryResult = sqlx::query!("DELETE FROM infractions WHERE id = $1", id)
        .execute(&ctx.data().database.pool)
        .await
        .unwrap();

    let res = match result.rows_affected() {
        1 => "Infraction deleted!",
        _ => "Infraction not deleted!",
    };

    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    guild_only,
    required_permissions = "KICK_MEMBERS | BAN_MEMBERS | MODERATE_MEMBERS"
)]
pub async fn user(ctx: Context<'_>, member: UserId) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;
    let user_id = member.get().to_string();

    if let Ok(user_infractions) = sqlx::query_as!(
        UserInfractionModel,
        r#"SELECT * FROM user_infractions WHERE user_id = $1"#,
        user_id,
    )
    .fetch_all(&ctx.data().database.pool)
    .await
    {
        let mut infractions_str = String::new();

        for infraction in user_infractions {
            let formatted = format_user_infraction(infraction);
            infractions_str.push_str(formatted.as_str());
        }

        let vec_pages: Vec<&str> = infractions_str.split("\r\n").collect();
        let pages: &[&str] = vec_pages.as_slice();
        poise::samples::paginate(ctx, pages).await?;
        return Ok(());
    }

    ctx.reply("User has no infractions!").await?;
    Ok(())
}

fn format_infraction(
    InfractionModel {
        id,
        severity,
        punishment,
        duration,
    }: InfractionModel,
) -> String {
    format!(
        "ID: {}\nSeverity: {:?}\nPunishment: {:?}\nDuration: {}\r\n",
        id, severity, punishment, duration
    )
}

fn format_user_infraction(
    UserInfractionModel {
        id,
        user_id,
        infraction_id,
        created_at,
    }: UserInfractionModel,
) -> String {
    format!(
        "<@{}> Case ID: {}\nInfraction ID: {}\nCreated at: {:?}\r\n",
        user_id, id, infraction_id, created_at
    )
}
