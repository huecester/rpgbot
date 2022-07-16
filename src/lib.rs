mod commands;
pub mod battle;
pub mod types;
pub mod util;

use crate::{
	commands::{duel, register},
	types::*,
};
use std::collections::HashSet;
use poise::{
	BoxFuture,
	serenity_prelude as serenity,
};

pub async fn start(token: impl Into<String>, owner_ids: Vec<impl Into<u64>>) -> Result<(), Error> {
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