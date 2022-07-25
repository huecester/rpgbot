use super::Weapon;

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;

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

	fn crit_ratio(&self) -> (u32, u32) { (7, 100) }
}