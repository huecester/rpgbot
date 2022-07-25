mod apple;
mod coin;
mod faulty_water_gun;
mod shield;

pub use apple::Apple;
pub use coin::Coin;
pub use faulty_water_gun::FaultyWaterGun;
pub use shield::Shield;

use crate::{
	prelude::*,
	battle::Battle,
};
use super::Player;

use async_trait::async_trait;
use poise::serenity_prelude::{CreateSelectMenuOption, ReactionType};
use uuid::Uuid;

#[async_trait]
pub trait Item: Send + Sync {
	fn new() -> Self where Self: Sized;
	fn name(&self) -> &str;
	fn id(&self) -> &Uuid;
	fn description(&self) -> &str;
	fn icon(&self) -> ReactionType;
	async fn use_item(&self, user: &Player, battle: &Battle, is_p1: bool) -> Result<(), Error>;
}

impl dyn Item {
	pub fn as_option<'a>(&self, o: &'a mut CreateSelectMenuOption) -> &'a mut CreateSelectMenuOption {
		o.label(self.name())
			.value(self.id())
			.description(self.description())
			.emoji(self.icon().clone())
	}
}
