use crate::{
	prelude::*,
	battle::{Battle, Battler},
	model::QueryItem,
};

use poise::serenity_prelude::{CreateSelectMenuOption, ReactionType};
use uuid::Uuid;

type ItemCallback = Box<dyn Fn(&Item, &mut dyn Battler, &mut Battle, &mut dyn Battler) + Send + Sync>;
pub struct Item {
	pub name: String,
	pub id: Uuid,
	pub description: String,
	pub icon: ReactionType,
	pub cb: ItemCallback,
}

impl Item {
	pub fn use_item(&self, user: &mut dyn Battler, battle: &mut Battle, opponent: &mut dyn Battler) {
		(self.cb)(self, user, battle, opponent);
	}

	pub fn as_option<'a>(&self, o: &'a mut CreateSelectMenuOption) -> &'a mut CreateSelectMenuOption {
		o.label(&self.name)
			.value(&self.id)
			.description(&self.description)
			.emoji(self.icon.clone())
	}
}

impl TryFrom<QueryItem> for Item {
	type Error = Error;

	fn try_from(item: QueryItem) -> Result<Self, Self::Error> {
		Ok(Self {
			name: item.name,
			id: Uuid::new_v4(),
			description: item.description,
			icon: item.icon.try_into()?,
			// TODO lua
			cb: Box::new(|_, _, _, _| {}),
		})
	}
}
