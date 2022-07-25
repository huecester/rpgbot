mod hammer;
mod spear;
mod sword;

pub use hammer::Hammer;
pub use spear::Spear;
pub use sword::Sword;

use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
		player::Player,
	},
};

use std::ops::RangeInclusive;
use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;
use rand::prelude::*;

#[async_trait]
pub trait Weapon: Send + Sync {
	fn new() -> Self where Self: Sized;
	fn name(&self) -> &str;
	fn icon(&self) -> ReactionType;

	fn damage_range(&self) -> RangeInclusive<usize> { 10..=20 }
	fn crit_ratio(&self) -> (u32, u32) { (2, 100) }
	fn crit_multiplier(&self) -> usize { 2 }
	fn pierce(&self) -> usize { 0 }
}

impl dyn Weapon {
	pub async fn attack(&self, user: &Player<'_>, battle: &Battle<'_>, is_p1: bool) -> Result<(), Error> {
		let mut log = battle.log.lock().await;

		let opponent = if is_p1 {
			battle.p2.lock().await
		} else {
			battle.p1.lock().await
		};

		let (mut damage, critical) = {
			let mut rand = rand::thread_rng();
			let damage: usize = rand.gen_range(self.damage_range());
			let (num, den) = self.crit_ratio();
			let critical = rand.gen_ratio(num, den);
			(damage, critical)
		};

		if critical {
			damage = damage.checked_mul(self.crit_multiplier()).unwrap_or(usize::MAX);
		}

		let damage_dealt = opponent.damage(damage, self.pierce());

		if critical {
			log.add(Entry::Critical(user.name().clone(), opponent.name().clone(), damage_dealt));
		} else {
			log.add(Entry::Attack(self.icon(), user.name().clone(), opponent.name().clone(), damage_dealt));
		}

		Ok(())
	}
}