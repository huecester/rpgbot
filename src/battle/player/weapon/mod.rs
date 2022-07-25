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
		player::Player,
	},
};

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;

#[async_trait]
pub trait Weapon: Send + Sync {
	fn new() -> Self where Self: Sized;
	fn name(&self) -> &str;
	fn icon(&self) -> ReactionType;
	async fn attack(&self, user: &Player, battle: &Battle, is_p1: bool) -> Result<(), Error>;
}