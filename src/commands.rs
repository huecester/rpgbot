use crate::{prelude::*, battle::Battle};
use poise::serenity_prelude as serenity;

/// Duel a user.
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = 10,
)]
pub async fn duel(
    ctx: Context<'_>,
    #[description = "User to duel."] opponent: serenity::User,
) -> Result<(), Error> {
    if opponent.bot {
        ctx.send(|r| r.content("You cannot challenge bots to a duel.").ephemeral(true)).await?;
        return Ok(());
    }

    if ctx.author() == &opponent {
        ctx.send(|r| r.content("You cannot challenge yourself to a duel.").ephemeral(true)).await?;
        return Ok(());
    }

	let p1 = ctx.author().clone();
	let mut battle = Battle::new(ctx, p1, opponent);
	if let Err(e) = battle.start().await {
        eprintln!("{:?}", e);
        return Err("There was an error during the battle.".into());
    };
	Ok(())
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
    if let Err(e) = poise::builtins::register_application_commands_buttons(ctx).await {
        eprintln!("{:?}", e);
        return Err("There was an error while registering commands.".into());
    };
    Ok(())
}
