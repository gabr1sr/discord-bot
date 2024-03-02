use crate::{Context, Error};
use crate::models::{Severity, Punishment, InfractionModel};

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "list", "remove"),
    subcommand_required,
    category = "Moderation"
)]
pub async fn infraction(_: Context<'_>) -> Result<(), Error> {
    Ok(())
}

#[poise::command(
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
    duration: i32
) -> Result<(), Error> {
    let insertion = sqlx::query_as!(
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

    let res = format!("{:?}", insertion);
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let result = sqlx::query_as!(
        InfractionModel,
        r#"SELECT id, severity AS "severity!: Severity", punishment AS "punishment!: Punishment", duration FROM infractions"#,
    )
        .fetch_all(&ctx.data().database.pool)
        .await
        .unwrap();

    let res = format!("{:?}", result);
    ctx.reply(res).await?;
    Ok(())
}

#[poise::command(
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn remove(ctx: Context<'_>, id: i32) -> Result<(), Error> {
    let result = sqlx::query!("DELETE FROM infractions WHERE id = $1", id)
        .execute(&ctx.data().database.pool)
        .await;

    let res = format!("{:?}", result);
    ctx.reply(res).await?;
    Ok(())
}
