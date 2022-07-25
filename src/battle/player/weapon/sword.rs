use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
		Player,
	}
};
use super::Weapon;

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;
use rand::prelude::*;

pub struct Sword;

#[async_trait]
impl Weapon for Sword {
	fn new() -> Self {
		Self
	}

	fn name(&self) -> &str {
		"Sword"
	}

	fn icon(&self) -> ReactionType {
		'⚔'.into()
	}

	async fn attack(&self, user: &Player, battle: &Battle, is_p1: bool) -> Result<(), Error> {
		let mut log = battle.log.lock().await;

		let opponent = if is_p1 {
			battle.p2.lock().await
		} else {
			battle.p1.lock().await
		};

		let (mut damage, critical) = {
			let mut rand = rand::thread_rng();
			let damage: usize = rand.gen_range(10..=20);
			let critical = rand.gen_ratio(5, 100);
			(damage, critical)
		};

		if critical {
			damage = damage.checked_mul(2).unwrap_or(usize::MAX);
		}

		let damage_dealt = opponent.damage(damage, 0);

		if critical {
			log.add(Entry::Critical(user.name().clone(), opponent.name().clone(), damage_dealt));
		} else {
			log.add(Entry::Attack(self.icon(), user.name().clone(), opponent.name().clone(), damage_dealt));
		}

		Ok(())
	}
}