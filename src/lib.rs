mod commands;
pub mod battle;
pub mod prelude;
pub mod types;
pub mod util;

use commands::{duel, register};
use prelude::*;

use std::collections::HashSet;
use poise::{
	BoxFuture,
	serenity_prelude as serenity,
};

pub async fn start<T, U>(token: T, owner_ids: Vec<U>) -> Result<(), Error>
    where
        T: Into<String> + Send,
        U: Into<u64> + Send,
{
    let mut owners = HashSet::new();
	for id in owner_ids {
		owners.insert(serenity::UserId(id.into()));
	}

    let framework = poise::Framework::build()
        .options(poise::FrameworkOptions{
            commands: vec![duel(), register()],
            owners,
            ..Default::default()
        })
        .token(token)
        .intents(serenity::GatewayIntents::non_privileged())
        .user_data_setup(move |_ctx, _ready, _framework| -> BoxFuture<'_, Result<Data, Error>> { Box::pin(async move { Ok(Data {}) }) });

    framework.run().await?;

	Ok(())
}