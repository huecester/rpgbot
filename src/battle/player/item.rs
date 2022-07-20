use crate::{
	prelude::*,
	battle::{Battler, Battle},
};

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
	async fn use_item<'a>(&self, user: &dyn Battler<'a>, battle: &Battle, is_p1: bool) -> Result<(), Error>;
}

impl dyn Item {
	pub fn as_option<'a>(&self, o: &'a mut CreateSelectMenuOption) -> &'a mut CreateSelectMenuOption {
		o.label(self.name())
			.value(self.id())
			.description(self.description())
			.emoji(self.icon().clone())
	}
}
