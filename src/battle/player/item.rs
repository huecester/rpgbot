use crate::{
	prelude::*,
	battle::{Battler, Battle, log::Entry},
};

use async_trait::async_trait;
use poise::serenity_prelude::{CreateSelectMenuOption, ReactionType};
use rand::Rng;
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

pub struct Apple(Uuid);

#[async_trait]
impl Item for Apple {
	fn new() -> Self {
		Self(Uuid::new_v4())
	}

	fn name(&self) -> &str {
		"Apple"
	}

	fn id(&self) -> &Uuid {
		&self.0
	}

	fn description(&self) ->  &str {
		"Heal 5-20 HP."
	}

	fn icon(&self) -> ReactionType {
		'🍎'.into()
	}

	async fn use_item<'a>(&self, user: &dyn Battler<'a>, battle: &Battle, _is_p1: bool) -> Result<(), Error> {
		let healing = rand::thread_rng().gen_range(5..=20);
		user.heal(healing);
		battle.log.lock().await.add(Entry::Item(self.icon(), format!("{} ate an apple and healed for {} health.", user.name(), healing)));
		Ok(())
	}
}