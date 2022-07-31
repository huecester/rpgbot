mod item;
mod util;
mod weapon;

pub use util::create_battle_embed;

use crate::{
	prelude::*,
	model::{QueryItem, QueryWeapon},
};
use super::{
	Battle,
	Battler,
	log::Entry,
	util::BattlerInfo,
};
use item::Item;
use util::create_battle_components;
use weapon::Weapon;

use std::{
	collections::HashMap,
	env,
};
use async_trait::async_trait;
use diesel::{
	prelude::*,
	pg::PgConnection,
};
use poise::serenity_prelude::{ButtonStyle, User, UserId };
use rand::prelude::*;
use uuid::Uuid;

pub struct Player<'a> {
	user: User,
	id: Uuid,
	is_p1: bool,
	ctx: Context<'a>,
	health: usize,
	max_health: usize,
	weapon: Weapon,
	items: HashMap<Uuid, Item>,
	armor: usize,
}

impl<'a> Player<'a> {
	pub fn new(user: User, ctx: Context<'a>, is_p1: bool) -> Result<Self, Error> {
		let (items, weapons) = {
			let database_url = env::var("DATABASE_URL")?;
			let conn = PgConnection::establish(&database_url)?;

			let items: Result<Vec<Item>, Error> = {
				use crate::schema::items::dsl::*;
				items.load::<QueryItem>(&conn)?.into_iter().map(|query_item| Item::try_from(query_item)).collect()
			};

			let weapons: Result<Vec<Weapon>, Error> = {
				use crate::schema::weapons::dsl::*;
				weapons.load::<QueryWeapon>(&conn)?.into_iter().map(|query_weapon| Weapon::try_from(query_weapon)).collect()
			};

			(items?, weapons?)
		};

		let items = items
			.into_iter()
			.flat_map(|item| vec![item.clone(), item])
			.choose_multiple(&mut rand::thread_rng(), 3)
			.into_iter()
			.fold(HashMap::new(), |mut acc, item| {
				acc.insert(item.id, item);
				acc
			});

		let weapon = weapons
			.into_iter()
			.choose(&mut rand::thread_rng())
			.ok_or("No weapons found.")?;

		Ok(Self {
			user,
			id: Uuid::new_v4(),
			is_p1,
			ctx,
			health: 100,
			max_health: 100,
			weapon,
			items,
			armor: 0,
		})
	}

	pub fn user(&self) -> &User {
		&self.user
	}

	pub fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
	}

	async fn act(&mut self, battle: &mut Battle<'_>, opponent: &mut dyn Battler) -> Result<(), Error> {
		loop {
			let self_display = self.info().display().await;
			let opponent_display = opponent.info().display().await;

			if self.is_p1 {
				battle.reply.edit(self.ctx, |m|
					m.embed(|e| create_battle_embed(e, &self_display, &opponent_display, battle.p1_turn, &battle.log))
						.components(|c| create_battle_components(c, false, self.items.is_empty()))
				).await?;
			} else {
				battle.reply.edit(self.ctx, |m|
					m.embed(|e| create_battle_embed(e, &opponent_display, &self_display, battle.p1_turn, &battle.log))
						.components(|c| create_battle_components(c, false, self.items.is_empty()))
				).await?;
			}

			let interaction = battle.reply
				.message()
				.await?
				.await_component_interaction(self.ctx.discord())
				.author_id(self.user.id)
				.await;

			if let Some(m) = interaction {
				m.defer(self.ctx.discord()).await?;

				match &*m.data.custom_id {
					"attack" => self.weapon.attack(self, battle, opponent),
					"surrender" => {
						battle.log.add(Entry::Surrender(self.name().clone()));
						self.set_health(0);
					},
					"item" => {
						if self.items.is_empty() {
							continue;
						}

						battle.reply.edit(self.ctx, |m|
							m.components(|c| create_battle_components(c, true, true))
						).await?;

						if !self.item(battle, opponent).await? {
							continue;
						}
					},
					other => return Err(format!("Unknown ID {other}.").into()),
				}
			} else {
				battle.log.add(Entry::Timeout(self.name().clone()));
			}

			break;
		}

		Ok(())
	}

	async fn item(&mut self, battle: &mut Battle<'_>, opponent: &mut dyn Battler) -> Result<bool, Error> {
		if self.items.is_empty() {
			return Ok(false);
		}

		let handle = self.ctx.send(|m|
			m.content("Select an item:")
				.components(|c|
					c.create_action_row(|r|
						r.create_select_menu(|m|
							m.custom_id("item")
								.placeholder("Select an item...")
								.options(|o| self.items.values().fold(o, |acc, item| acc.create_option(|o| item.as_option(o))))
						)
					).create_action_row(|r|
						r.create_button(|b|
							b.custom_id("back")
								.emoji('◀')
								.label("Back")
								.style(ButtonStyle::Danger)
						)
					)
				)
		).await?;
		let message = handle.message().await?;

		let interaction = message
			.await_component_interaction(self.ctx.discord())
			.author_id(self.user.id)
			.await;

		message.delete(self.ctx.discord()).await?;

		if let Some(m) = interaction {
			m.defer(self.ctx.discord()).await?;

			match &*m.data.custom_id {
				"item" => {
					let item_id = Uuid::parse_str(m.data.values.get(0).ok_or("No values received.")?)?;
					let item = self.items
						.remove(&item_id)
						.ok_or(format!("Item ID {} not found.", item_id))?;

					item.use_item(self, battle, opponent)?;
				},
				"back" => {
					return Ok(false);
				},
				other => return Err(format!("Unknown ID {other}.").into()),
			}
		} else {
			return Ok(false);
		}

		Ok(true)
	}
}

#[async_trait]
impl<'a> Battler for Player<'a> {
	fn user_id(&self) -> Option<UserId> {
		Some(self.user.id)
	}
	fn id(&self) -> &Uuid {
		&self.id
	}
	fn name(&self) -> &String {
		&self.user.name
	}
	fn icon(&self) -> Option<String> {
		self.user.avatar_url()
	}

	async fn act(&mut self, battle: &mut Battle, opponent: &mut dyn Battler) -> Result<(), Error> {
		self.act(battle, opponent).await
	}

	fn health(&self) -> usize {
		self.health
	}
	fn max_health(&self) -> usize {
		self.max_health
	}
	fn armor(&self) -> usize {
		self.armor
	}

	fn set_health(&mut self, health: usize) {
		self.health = health.clamp(0, self.max_health);
	}
	fn set_armor(&mut self, armor: usize) {
		self.armor = armor;
	}

	fn info(&self) -> BattlerInfo {
		BattlerInfo {
			ctx: self.ctx,
			name: self.name().clone(),
			icon: self.icon(),
			health: self.health(),
			max_health: self.max_health(),
			weapon: (self.weapon.icon.clone(), self.weapon.name.clone()),
			armor: self.armor,
		}
	}
}