use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
	},
	model::QueryItem,
};

use std::cell::RefCell;
use poise::serenity_prelude::{CreateSelectMenuOption, ReactionType};
use rlua::Lua;
use uuid::Uuid;

#[derive(Clone)]
pub struct Item {
	pub name: String,
	pub id: Uuid,
	pub description: String,
	pub icon: ReactionType,
	pub lua: String,
}

impl Item {
	pub fn use_item<'a>(&self, user: &mut dyn Battler, battle: &mut Battle, opponent: &mut dyn Battler) -> Result<(), Error> {
		let icon = self.icon.clone();

		let user = RefCell::new(user);
		let opponent = RefCell::new(opponent);

		let lua = Lua::new();
		lua.context(|lua_ctx| -> Result<(), Error> {
			lua_ctx.scope(|scope| -> Result<(), Error> {
				let globals = lua_ctx.globals();

				globals.set("user_name", user.borrow().name().clone())?;
				globals.set("opponent_name", opponent.borrow().name().clone())?;

				let add_log_entry = scope.create_function_mut(|_, entry: String| {
					battle.log.add(Entry::Item(icon.clone(), entry));
					Ok(())
				})?;
				globals.set("add_log_entry", add_log_entry)?;

				let heal_user = scope.create_function_mut(|_, healing: usize| {
					Ok(user.borrow_mut().heal(healing))
				})?;
				globals.set("heal_user", heal_user)?;

				let heal_opponent = scope.create_function_mut(|_, healing: usize| {
					Ok(opponent.borrow_mut().heal(healing))
				})?;
				globals.set("heal_opponent", heal_opponent)?;

				let damage_user = scope.create_function_mut(|_, (damage, pierce): (usize, usize)| {
					Ok(user.borrow_mut().damage(damage, pierce))
				})?;
				globals.set("damage_user", damage_user)?;

				let damage_opponent = scope.create_function_mut(|_, (damage, pierce): (usize, usize)| {
					Ok(opponent.borrow_mut().damage(damage, pierce))
				})?;
				globals.set("damage_opponent", damage_opponent)?;

				let add_armor = scope.create_function_mut(|_, armor: usize| {
					user.borrow_mut().add_armor(armor);
					Ok(())
				})?;
				globals.set("add_armor", add_armor)?;

				lua_ctx
					.load(&format!("math.randomseed(os.time())\n{}", self.lua))
					.set_name(&format!("{}", self.name))?
					.exec()?;

				Ok(())
			})?;

			Ok(())
		})?;

		Ok(())
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
			lua: item.lua,
		})
	}
}
