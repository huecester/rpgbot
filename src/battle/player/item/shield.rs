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

use std::sync::atomic::Ordering;
use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;
use rand::Rng;
use uuid::Uuid;

pub struct Shield(Uuid);

#[async_trait]
impl Item for Shield {
	fn new() -> Self {
		Self(Uuid::new_v4())
	}

	fn name(&self) -> &str {
		"Shield"
	}

	fn id(&self) -> &Uuid {
		&self.0
	}

	fn description(&self) -> &str {
		"Gain 5-10 armor."
	}

	fn icon(&self) -> ReactionType {
		'🛡'.into()
	}

	async fn use_item(&self, player: &Player, battle: &Battle, _is_p1: bool) -> Result<(), Error> {
		let armor = {
			let mut rng = rand::thread_rng();
			rng.gen_range(5..=10)
		};
		let mut log = battle.log.lock().await;

		let current_armor = player.armor.load(Ordering::Relaxed);
		player.armor.store(current_armor.checked_add(armor).unwrap_or(usize::MAX), Ordering::Relaxed);
		log.add(Entry::Item(self.icon(), format!("{} equipped a shield, gaining {} armor.", player.name(), armor)));

		Ok(())
	}
}