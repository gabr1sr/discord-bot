use sqlx::postgres::PgQueryResult;

use crate::models::{InfractionModel, Punishment, Severity};
use crate::{Context, Error};

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "list", "remove"),
    subcommand_required,
    required_permissions = "ADMINISTRATOR",
    category = "Moderation"
)]
pub async fn infraction(_: Context<'_>) -> Result<(), Error> {
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
    duration: i32,
) -> Result<(), Error> {
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
