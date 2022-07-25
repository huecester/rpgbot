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

pub struct Coin(Uuid);

#[async_trait]
impl Item for Coin {
	fn new() -> Self {
		Self(Uuid::new_v4())
	}

	fn name(&self) -> &str {
		"Coin"
	}

	fn id(&self) -> &Uuid {
		&self.0
	}

	fn description(&self) -> &str {
		"50/50 chance to heal/hurt your opponent for 20-35 health."
	}

	fn icon(&self) -> ReactionType {
		'🪙'.into()
	}

	async fn use_item(&self, player: &Player, battle: &Battle, is_p1: bool) -> Result<(), Error> {
		let (health, heal) = {
			let mut rng = rand::thread_rng();
			let health = rng.gen_range(20..=35);
			let heal = rng.gen();
			(health, heal)
		};
		let mut log = battle.log.lock().await;

		let opponent = if is_p1 { battle.p2.lock().await } else { battle.p1.lock().await };
		if heal {
			let healing = opponent.heal(health);
			log.add(Entry::Item(self.icon(), format!("{} flipped {} healing against {}.", player.name(), healing, opponent.name())));
		} else {
			let damage = opponent.damage(health);
			log.add(Entry::Item(self.icon(), format!("{} flipped {} damage against {}.", player.name(), damage, opponent.name())))
		}

		Ok(())
	}
}