use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
	},
};
use super::{item::Item, Player};

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
		player.heal(healing);
		battle.log.lock().await.add(Entry::Item(self.icon(), format!("{} ate an apple and healed for {} health.", player.name(), healing)));
		Ok(())
	}
}


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
			opponent.heal(health);
			log.add(Entry::Item(self.icon(), format!("{} flipped {} healing against {}.", player.name(), health, opponent.name())));
		} else {
			opponent.damage(health);
			log.add(Entry::Item(self.icon(), format!("{} flipped {} damage against {}.", player.name(), health, opponent.name())))
		}

		Ok(())
	}
}


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
			player.damage(self_damage);
			log.add(Entry::Item(self.icon(), format!("{}'s water gun backfired, dealing {} damage to themselves.", player.name(), self_damage)));
		} else {
			let opponent = if is_p1 { battle.p2.lock().await } else { battle.p1.lock().await };
			opponent.damage(opponent_damage);
			log.add(Entry::Item(self.icon(), format!("{} splashed {} with a water gun, dealing {} damage.", player.name(), opponent.name(), opponent_damage)));
		}

		Ok(())
	}
}