use std::{
    env,
    error,
};
use dotenv::dotenv;
use poise::serenity_prelude as serenity;

type Error = Box<dyn error::Error + Send + Sync>;
type Context<'a> = poise::Context<'a, Data, Error>;
struct Data {}

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

#[poise::command(prefix_command)]
async fn register(ctx: Context<'_>) -> Result<(), Error> {
    poise::builtins::register_application_commands_buttons(ctx).await?;
    Ok(())
}

#[tokio::main]
async fn main() {
    dotenv().unwrap();

    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions{
            commands: vec![age(), register()],
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found."))
        .intents(serenity::GatewayIntents::GUILDS)
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));
    
    framework.run().await.unwrap();
}
