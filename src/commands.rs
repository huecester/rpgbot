use crate::{
    battle::Battle,
    prelude::*,
};
use poise::serenity_prelude::User;

/// Duel a user.
#[poise::command(
    slash_command,
    guild_only,
    user_cooldown = 10,
)]
pub async fn duel(
    ctx: Context<'_>,
    #[description = "User to duel."] opponent: User,
) -> Result<(), Error> {
    let p1 = ctx.author().clone();

    if opponent.bot {
        ctx.send(|m| m.content("You cannot challenge bots to a duel.").ephemeral(true)).await?;
        return Ok(());
    }

    if p1 == opponent {
        ctx.send(|m| m.content("You cannot challenge yourself to a duel.").ephemeral(true)).await?;
        return Ok(());
    }

    let data = ctx.data();
    if data.check_for_user_in_battle(&p1) {
        ctx.send(|m| m.content("You cannot be in two battles at once.").ephemeral(true)).await?;
        return Ok(());
    }
    if data.check_for_user_in_battle(&opponent) {
        ctx.send(|m| m.content("That user is currently in a battle. Try again later.").ephemeral(true)).await?;
        return Ok(());
    }

	if let Err(e) = Battle::send_invite(ctx, p1, opponent).await {
        eprintln!("{:?}", e);
        ctx.send(|m| m.content("There was an error during the battle.").ephemeral(true)).await?;
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
