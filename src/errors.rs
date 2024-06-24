use crate::{Context, Data, Error};
use std::time::Duration;

use poise::{
    serenity_prelude::{CommandInteraction, EmojiParseError, FullEvent, Message, Permissions},
    ApplicationContext, CreateReply,
};

pub mod argument_parse;

pub async fn on_error(error: poise::FrameworkError<'_, Data, Error>) {
    match error {
        poise::FrameworkError::Setup { error, .. } => handle_setup_error(error),
        poise::FrameworkError::EventHandler { error, event, .. } => {
            handle_event_handler_error(error, event)
        }
        poise::FrameworkError::Command { error, ctx, .. } => handle_command_error(error, ctx),
        poise::FrameworkError::SubcommandRequired { ctx } => {
            if let Err(error) = handle_subcommand_required_error(ctx).await {
                println!("Failed to handle subcommand required error: {:?}", error);
            }
        }
        poise::FrameworkError::CommandPanic { payload, ctx, .. } => {
            if let Err(error) = handle_command_panic_error(payload, ctx).await {
                println!("Failed to handle command panic error: {:?}", error);
            }
        }
        poise::FrameworkError::ArgumentParse {
            error, input, ctx, ..
        } => {
            if let Err(error) = handle_argument_parse_error(error, input, ctx).await {
                println!("Failed to handle argument parse error: {:?}", error);
            }
        }
        poise::FrameworkError::CommandStructureMismatch {
            description, ctx, ..
        } => handle_command_structure_mismatch_error(description, ctx),
        poise::FrameworkError::CooldownHit {
            remaining_cooldown,
            ctx,
            ..
        } => {
            if let Err(error) = handle_cooldown_hit_error(remaining_cooldown, ctx).await {
                println!("Failed to handle cooldown hit error: {:?}", error);
            }
        }
        poise::FrameworkError::MissingBotPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            if let Err(error) = handle_missing_bot_permissions_error(missing_permissions, ctx).await
            {
                println!(
                    "Failed to handle missing bot permissions error: {:?}",
                    error
                );
            }
        }
        poise::FrameworkError::MissingUserPermissions {
            missing_permissions,
            ctx,
            ..
        } => {
            if let Err(error) =
                handle_missing_user_permissions_error(missing_permissions, ctx).await
            {
                println!(
                    "Failed to handle missing user permissions error: {:?}",
                    error
                );
            }
        }
        poise::FrameworkError::NotAnOwner { ctx, .. } => {
            if let Err(error) = handle_not_an_owner_error(ctx).await {
                println!("Failed to handle not an owner error: {:?}", error);
            }
        }
        poise::FrameworkError::GuildOnly { ctx, .. } => {
            if let Err(error) = handle_guild_only_error(ctx).await {
                println!("Failed to handle guild only error: {:?}", error);
            }
        }
        poise::FrameworkError::DmOnly { ctx, .. } => {
            if let Err(error) = handle_dm_only_error(ctx).await {
                println!("Failed to handle dm only error: {:?}", error);
            }
        }
        poise::FrameworkError::NsfwOnly { ctx, .. } => {
            if let Err(error) = handle_nsfw_only_error(ctx).await {
                println!("Failed to handle nsfw only error: {:?}", error);
            }
        }
        poise::FrameworkError::CommandCheckFailed { error, ctx, .. } => {
            if let Err(error) = handle_command_check_failed_error(error, ctx).await {
                println!("Failed to handle command check error: {:?}", error);
            }
        }
        poise::FrameworkError::DynamicPrefix { error, msg, .. } => {
            handle_dynamic_prefix_error(error, msg)
        }
        poise::FrameworkError::UnknownCommand {
            prefix,
            msg_content,
            ..
        } => handle_unknown_command_error(msg_content, prefix),
        poise::FrameworkError::UnknownInteraction { interaction, .. } => {
            handle_unknown_interaction_error(interaction)
        }
        poise::FrameworkError::__NonExhaustive(unreachable) => match unreachable {},
    }
}

fn handle_setup_error(error: Error) {
    panic!("Failed to start bot: {:?}", error);
}

fn handle_event_handler_error(error: Error, event: &FullEvent) {
    println!("Error in event `{}`: {:?}", event.snake_case_name(), error);
}

fn handle_command_error(error: Error, ctx: Context<'_>) {
    println!("Error in command `{}`: {:?}", ctx.command().name, error);
}

async fn handle_subcommand_required_error(ctx: Context<'_>) -> Result<(), Error> {
    println!(
        "User forgot to specify a subcommand for `{}` command!",
        &ctx.command().name
    );

    let subcommands: Vec<&str> = ctx.command().subcommands.iter().map(|s| &*s.name).collect();

    let res = format!(
        ":warning: You must specify one of the following subcommands: {}",
        subcommands.join(", ")
    );

    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_command_panic_error(
    payload: Option<String>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    println!(
        "Command `{}` has panicked! Payload: {:?}",
        &ctx.command().name,
        payload.unwrap()
    );

    let res = format!(":x: An unexpected internal error has occurred.");
    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_argument_parse_error(
    error: Error,
    input: Option<String>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    println!(
        "Error in argument parse (payload = {:?}) in command `{}`: {:?}!",
        input,
        &ctx.command().name,
        error
    );

    let mut res = String::new();

    // TODO: make argument parse error handler send message back to user
    // downcast errors, match them and handle into different functions
    // add a new handle when creating a new command
    if let Ok(error) = error.downcast::<EmojiParseError>() {
        res = argument_parse::handle_emoji_parse_error(error, input, ctx).await?;
    }

    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

fn handle_command_structure_mismatch_error(
    description: &str,
    ctx: ApplicationContext<'_, Data, Error>,
) {
    println!(
        "Structure mismatch in command `{}`: {}",
        ctx.command.name, description
    );
}

async fn handle_cooldown_hit_error(
    remaining_cooldown: Duration,
    ctx: Context<'_>,
) -> Result<(), Error> {
    let user = &ctx.author().tag();
    let seconds = remaining_cooldown.as_secs();

    println!(
        "User `{}` hit a cooldown. He must wait {} seconds before retrying.",
        user, seconds
    );

    let res = format!(
        ":warning: Cooldown hit! Please wait {} seconds before trying again!",
        seconds
    );

    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_missing_bot_permissions_error(
    missing_permissions: Permissions,
    ctx: Context<'_>,
) -> Result<(), Error> {
    println!(
        "Cannot execute command `{}` because is missing permissions: {}",
        ctx.command().name,
        missing_permissions
    );

    let res = format!(
        ":x: Command couldn't be executed because bot is lacking permissions: {}",
        missing_permissions
    );

    let builder = CreateReply::default().content(res);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_missing_user_permissions_error(
    missing_permissions: Option<Permissions>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    println!(
        "Cannot execute command `{}` because user `{}` is missing permissions: {}",
        ctx.command().name,
        ctx.author().tag(),
        missing_permissions.unwrap()
    );

    let res = if let Some(missing_permissions) = missing_permissions {
        format!(
            ":x: Command couldn't be executed because you are lacking permissions: {}",
            missing_permissions
        )
    } else {
        format!(":x: Command couldn't be executed because you are lacking permissions.")
    };

    let builder = CreateReply::default().content(res);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_not_an_owner_error(ctx: Context<'_>) -> Result<(), Error> {
    println!(
        "User `{}` tried to execute command `{}` that is only callable by this bot owners!",
        ctx.author().tag(),
        ctx.command().name
    );

    let res = format!(":x: Sorry, but this command is only callable by bot owners!");
    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_guild_only_error(ctx: Context<'_>) -> Result<(), Error> {
    println!(
        "User `{}` tried to execute command `{}` in DMs, but that is a guild only command.",
        ctx.author().tag(),
        ctx.command().name
    );

    let res = format!(":x: Sorry, but this command can only be executed inside servers.");
    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_dm_only_error(ctx: Context<'_>) -> Result<(), Error> {
    println!(
        "User `{}` tried to execute command `{}` inside a guild, but that is a DMs only command.",
        ctx.author().tag(),
        ctx.command().name
    );

    let res = format!(":x: Sorry, but this command can only be executed in DMs.");
    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_nsfw_only_error(ctx: Context<'_>) -> Result<(), Error> {
    println!(
        "User `{}` tried to execute command `{}`, but that is a NSFW channels only command.",
        ctx.author().tag(),
        ctx.command().name
    );

    let res = format!(":x: Sorry, but this command can only be executed in NSFW channels.");
    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

async fn handle_command_check_failed_error(
    error: Option<Error>,
    ctx: Context<'_>,
) -> Result<(), Error> {
    println!(
        "Command `{}` check failed for user `{}`: {:?}",
        ctx.command().name,
        ctx.author().tag(),
        error,
    );

    let res = format!(":x: A command check failed: {}", error.unwrap());
    let builder = CreateReply::default().content(res).ephemeral(true);
    ctx.send(builder).await?;
    Ok(())
}

fn handle_dynamic_prefix_error(error: Error, msg: &Message) {
    println!(
        "Dynamic prefix failed for message {:?}: {:?}",
        msg.content, error
    );
}

fn handle_unknown_command_error(msg_content: &str, prefix: &str) {
    println!(
        "Recognized prefix `{}`, but didn't recognized command name in `{}`",
        prefix, msg_content
    );
}

fn handle_unknown_interaction_error(interaction: &CommandInteraction) {
    println!("Received an unknown interaction: {:?}", interaction);
}
