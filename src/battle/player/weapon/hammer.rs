use std::ops::RangeInclusive;

use super::Weapon;

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;

pub struct Hammer;

#[async_trait]
impl Weapon for Hammer {
	fn new() -> Self {
		Self
	}

	fn name(&self) -> &str {
		"Hammer"
	}

	fn icon(&self) -> ReactionType {
		'🔨'.into()
	}

	fn damage_range(&self) -> RangeInclusive<usize> { 15..=30 }
}