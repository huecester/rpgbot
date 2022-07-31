use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
	},
	model::QueryWeapon,
};

use std::ops::{
	Bound,
	RangeInclusive,
};
use poise::serenity_prelude::ReactionType;
use rand::prelude::*;

#[derive(Clone)]
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

impl TryFrom<QueryWeapon> for Weapon {
	type Error = Error;

	fn try_from(weapon: QueryWeapon) -> Result<Self, Self::Error> {
		let damage_range = if let Some(damage_range) = weapon.damage_range {
			match damage_range {
				(Bound::Included(lower), Bound::Included(higher)) => (lower.max(0).try_into()?)..=(higher.max(0).try_into()?),
				(Bound::Included(lower), Bound::Excluded(higher)) => (lower.max(0).try_into()?)..=(higher.saturating_sub(1).max(0).try_into()?),
				(Bound::Excluded(lower), Bound::Included(higher)) => (lower.saturating_add(1).max(0).try_into()?)..=(higher.max(0).try_into()?),
				(Bound::Excluded(lower), Bound::Excluded(higher)) => (lower.saturating_add(1).max(0).try_into()?)..=(higher.saturating_sub(1).max(0).try_into()?),
				_ => return Err("Range cannot be open.".into()),
			}
		} else {
			10..=20
		};

		Ok(Self {
			name: weapon.name,
			icon: weapon.icon.try_into()?,
			damage_range,
			crit_ratio: weapon.crit_ratio.unwrap_or(2.0 / 100.0),
			crit_multiplier: weapon.crit_multiplier.unwrap_or(2).max(0).try_into()?,
			pierce: weapon.pierce.unwrap_or(0).max(0).try_into()?,
		})
	}
}