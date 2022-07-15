use crate::types::*;
use poise::serenity_prelude as serenity;

/// Duel a user.
#[poise::command(
    slash_command,
    guild_only,
)]
pub async fn duel(
    ctx: Context<'_>,
    #[description = "User to duel."] opponent: serenity::User,
) -> Result<(), Error> {
	todo!()
}

/// Displays a menu for registering slash commands.
#[poise::command(
    prefix_command,
    owners_only,
    hide_in_help,
    guild_only,
    reuse_response,
)]
pub async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}
