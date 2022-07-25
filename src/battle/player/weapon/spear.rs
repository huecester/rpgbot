use super::Weapon;

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;

pub struct Spear;

#[async_trait]
impl Weapon for Spear {
	fn new() -> Self {
		Self
	}

	fn name(&self) -> &str {
		"Spear"
	}

	fn icon(&self) -> ReactionType {
		'⚔'.into()
	}

	fn pierce(&self) -> usize { 5 }
}