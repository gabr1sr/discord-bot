use crate::models::{InfractionModel, Punishment, Severity};
use crate::{Context, Error};
use serenity::model::id::UserId;

#[poise::command(
    slash_command,
    prefix_command,
    subcommands("add", "list", "remove", "user", "edit"),
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

    if let Ok(_) = ctx.data().database.get_infraction(id).await {
        ctx.reply(format!(":warning: Infraction ID `{id}` already exists!"))
            .await?;
        return Ok(());
    }

    if let Ok(infraction) = ctx
        .data()
        .database
        .add_infraction(id, severity, punishment, duration)
        .await
    {
        let data = format_infraction(infraction);
        ctx.reply(format!(":white_check_mark: Infraction created!\n{data}"))
            .await?;
        return Ok(());
    }

    ctx.reply(format!(":x: Failed to create infraction ID `{id}`!"))
        .await?;
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

    if let Ok(infractions) = ctx.data().database.get_infractions().await {
        let res = if infractions.is_empty() {
            "No infractions in table!".to_owned()
        } else {
            infractions
                .iter()
                .map(|i| {
                    format!(
                        "- ID: `{}` | Severity: `{:?}` | Punishment: `{:?}` | Duration: `{}`",
                        i.id, i.severity, i.punishment, i.duration
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        };

        ctx.reply(res).await?;
        return Ok(());
    }

    ctx.reply(format!("No infractions found!")).await?;
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

    if let Ok(result) = ctx.data().database.remove_infraction(id).await {
        let res = match result.rows_affected() {
            0 => "No infraction removed!".to_owned(),
            1 => "Infraction removed successfully!".to_owned(),
            _ => "Infractions removed successfully!".to_owned(),
        };

        ctx.reply(res).await?;
        return Ok(());
    }

    ctx.reply(format!("Failed to remove infraction!")).await?;
    Ok(())
}

#[poise::command(
    ephemeral,
    slash_command,
    prefix_command,
    required_permissions = "ADMINISTRATOR",
    guild_only
)]
pub async fn edit(
    ctx: Context<'_>,
    id: i32,
    severity: Severity,
    punishment: Punishment,
    duration: i64,
) -> Result<(), Error> {
    ctx.defer_ephemeral().await?;

    if let Err(_) = ctx.data().database.get_infraction(id).await {
        ctx.reply("This infraction doesn't exists!").await?;
        return Ok(());
    }

    if let Ok(_) = ctx
        .data()
        .database
        .update_infraction(id, severity, punishment, duration)
        .await
    {
        ctx.reply("Infraction updated with success!").await?;
        return Ok(());
    }

    ctx.reply("Failed to edit infraction!").await?;
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

    if let Ok(infractions) = ctx.data().database.get_user_infractions(member).await {
        let res = if infractions.is_empty() {
            "User has no infractions!".to_owned()
        } else {
            infractions
                .iter()
                .map(|i| {
                    format!(
                        "- ID: `{}` | User ID: `{}` | Infraction ID: `{}` | Created At: `{:?}`",
                        i.id, i.user_id, i.infraction_id, i.created_at
                    )
                })
                .collect::<Vec<String>>()
                .join("\n")
        };

        ctx.reply(res).await?;
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
        ":information_source: ID: {}\n:information_source: Severity: {:?}\n:information_source: Punishment: {:?}\n:information_source: Duration: {}\r\n",
        id, severity, punishment, duration
    )
}
