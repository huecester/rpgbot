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

pub struct FaultyWaterGun(Uuid);

#[async_trait]
impl Item for FaultyWaterGun {
	fn new() -> Self {
		Self(Uuid::new_v4())
	}

	fn name(&self) -> &str {
		"Faulty Water Gun"
	}

	fn id(&self) -> &Uuid {
		&self.0
	}

	fn description(&self) -> &str {
		"90% chance to deal 30-40 damage; 10% chance to backfire for 50-60 damage"
	}

	fn icon(&self) -> ReactionType {
		'🔫'.into()
	}

	async fn use_item(&self, player: &Player, battle: &Battle, is_p1: bool) -> Result<(), Error> {
		let (opponent_damage, backfire, self_damage) = {
			let mut rng = rand::thread_rng();
			let opponent_damage = rng.gen_range(30..=40);
			let backfire = rng.gen_ratio(1, 10);
			let self_damage = rng.gen_range(50..=60);
			(opponent_damage, backfire, self_damage)
		};
		let mut log = battle.log.lock().await;

		if backfire {
			let damage = player.damage(self_damage, 0);
			log.add(Entry::Item(self.icon(), format!("{}'s water gun backfired, dealing {} damage to themselves.", player.name(), damage)));
		} else {
			let opponent = if is_p1 { battle.p2.lock().await } else { battle.p1.lock().await };
			let damage = opponent.damage(opponent_damage, 0);
			log.add(Entry::Item(self.icon(), format!("{} splashed {} with a water gun, dealing {} damage.", player.name(), opponent.name(), damage)));
		}

		Ok(())
	}
}
