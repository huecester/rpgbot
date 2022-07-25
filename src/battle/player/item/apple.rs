use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
		player::Player,
	},
};
use super::Item;

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;
use rand::Rng;
use uuid::Uuid;

pub struct Apple(Uuid);

#[async_trait]
impl Item for Apple {
	fn new() -> Self {
		Self(Uuid::new_v4())
	}

	fn name(&self) -> &str {
		"Apple"
	}

	fn id(&self) -> &Uuid {
		&self.0
	}

	fn description(&self) -> &str {
		"Heal 5-20 HP."
	}

	fn icon(&self) -> ReactionType {
		'🍎'.into()
	}

	async fn use_item(&self, player: &Player, battle: &Battle, _is_p1: bool) -> Result<(), Error> {
		let healing = rand::thread_rng().gen_range(5..=20);
		let healing = player.heal(healing);
		battle.log.lock().await.add(Entry::Item(self.icon(), format!("{} ate an apple and healed for {} health.", player.name(), healing)));
		Ok(())
	}
}