use crate::battle::{
	Battle,
	Battler,
	log::Entry,
};

use std::ops::RangeInclusive;
use poise::serenity_prelude::ReactionType;
use rand::prelude::*;

pub struct Weapon {
	pub name: String,
	pub icon: ReactionType,
	pub damage_range: RangeInclusive<usize>,
	pub crit_ratio: f64,
	pub crit_multiplier: usize,
	pub pierce: usize,
}

impl Weapon {
	pub fn attack(&self, user: &dyn Battler, battle: &mut Battle, opponent: &mut dyn Battler) {
		let mut rand = rand::thread_rng();
		let mut damage = rand.gen_range(self.damage_range.clone());
		let critical = rand.gen_bool(self.crit_ratio);

		if critical {
			damage = damage.checked_mul(self.crit_multiplier).unwrap_or(usize::MAX);
		}

		let damage_dealt = opponent.damage(damage, self.pierce);

		if critical {
			battle.log.add(Entry::Critical(user.name().clone(), opponent.name().clone(), damage_dealt));
		} else {
			battle.log.add(Entry::Attack(self.icon.clone(), user.name().clone(), opponent.name().clone(), damage_dealt));
		}
	}
}

impl Default for Weapon {
	fn default() -> Self {
		Self {
			name: "".into(),
			icon: '⚔'.into(),
			damage_range: 10..=20,
			crit_ratio: 2.0 / 100.0,
			crit_multiplier: 2,
			pierce: 0,
		}
	}
}