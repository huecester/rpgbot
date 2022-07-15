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

/// Duel a user.
#[poise::command(
    slash_command,
    guild_only,
)]
async fn duel(
    ctx: Context<'_>,
    #[description = "User to duel."] opponent: serenity::User,
) -> Result<(), Error> {
    let mut reply = ctx.send(|m|
        m.components(|c|
            c.create_action_row(|r|
                r.create_button(|b|
                    b.custom_id("fight")
                        .label("⚔ Fight")
                        .style(serenity::ButtonStyle::Primary)
                ).create_button(|b|
                    b.custom_id("run")
                        .label("💨 Run")
                        .style(serenity::ButtonStyle::Danger)
                )
            )
        )
    ).await?.message().await?;

    let interaction = reply.
        await_component_interaction(ctx.discord())
        .author_id(opponent.id)
        .await;
    
    reply.edit(ctx.discord(), |b| b.components(|b| b)).await?;

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
            commands: vec![duel(), register()],
            owners,
            ..Default::default()
        })
        .token(env::var("DISCORD_TOKEN").expect("DISCORD_TOKEN not found."))
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| Box::pin(async move { Ok(Data {}) }));
    
    framework.run().await.unwrap();
}
