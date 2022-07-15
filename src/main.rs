use std::{
    collections::HashSet,
    env,
    error,
};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

type Error = Box<dyn error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
struct Data {}

/// Queries a user's age.
#[poise::command(slash_command)]
async fn age(
    ctx: Context<'_>,
    #[description = "Selected user"] user: Option<serenity::User>,
) -> Result<(), Error> {
    let u = user.as_ref().unwrap_or_else(|| ctx.author());
    let response = format!("{}'s account was created on {}.", u.name, u.created_at());
    ctx.say(response).await?;
    Ok(())
}

/// Displays a menu for registering slash commands.
#[poise::command(
    prefix_command,
    slash_command,
    owners_only,
    hide_in_help,
    guild_only,
    reuse_response,
)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();

    let mut owners = HashSet::new();
    owners.insert(serenity::UserId(297860975971926017));

    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions{
            commands: vec![age(), register()],
            owners,
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found."))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));
    
    framework.run().await.unwrap();
}
