mod item;
mod util;
mod weapon;

pub use util::create_battle_embed;

use crate::prelude::*;
use super::{
	Battle,
	Battler,
	log::Entry,
	util::BattlerInfo,
};
use item::Item;
use util::create_battle_components;
use weapon::Weapon;

use std::collections::HashMap;
use async_trait::async_trait;
use poise::serenity_prelude::{ButtonStyle, User, UserId };
use rand::prelude::*;
use uuid::Uuid;

fn create_items() -> Vec<Item> {
	vec![
		Item {
			name: "Apple".to_string(),
			id: Uuid::new_v4(),
			description: "Heal 5-20 HP.".to_string(),
			icon: '🍎'.into(),
			cb: Box::new(|item, user, battle, _| {
				let healing = rand::thread_rng().gen_range(5..=20);
				let healing = user.heal(healing);
				battle.log.add(Entry::Item(item.icon.clone(), format!("{} ate an apple and healed for {} health.", user.name(), healing)));
			}),
		},
		Item {
			name: "Coin".to_string(),
			id: Uuid::new_v4(),
			description: "50/50 chance to heal/hurt your opponent for 20-35 health.".to_string(),
			icon: '🪙'.into(),
			cb: Box::new(|item, user, battle, opponent| {
				let mut rng = rand::thread_rng();
				let heal = rng.gen();
				let health = rng.gen_range(20..=35);
				if heal {
					let healing = opponent.heal(health);
					battle.log.add(Entry::Item(item.icon.clone(), format!("{} flipped {} healing against {}.", user.name(), healing, opponent.name())));
				} else {
					let damage = opponent.damage(health, 0);
					battle.log.add(Entry::Item(item.icon.clone(), format!("{} flipped {} damage against {}.", user.name(), damage, opponent.name())));
				}
			}),
		},
		Item {
			name: "Faulty Water Gun".to_string(),
			id: Uuid::new_v4(),
			description: "90% chance to deal 30-40 damage; 10% chance to backfire for 50-60 damage".to_string(),
			icon: '🔫'.into(),
			cb: Box::new(|item, user, battle, opponent| {
				let mut rng = rand::thread_rng();
				let opponent_damage = rng.gen_range(30..=40);
				let backfire = rng.gen_ratio(1, 10);
				let self_damage = rng.gen_range(50..=60);

				if backfire {
					let damage = user.damage(self_damage, 0);
					battle.log.add(Entry::Item(item.icon.clone(), format!("{}'s water gun backfired, dealing {} damage to themselves.", user.name(), damage)));
				} else {
					let damage = opponent.damage(opponent_damage, 0);
					battle.log.add(Entry::Item(item.icon.clone(), format!("{} splashed {} with a water gun, dealing {} damage.", user.name(), opponent.name(), damage)));
				}
			}),
		},
		Item {
			name: "Shield".to_string(),
			id: Uuid::new_v4(),
			description: "Gain 5-10 armor.".to_string(),
			icon: '🛡'.into(),
			cb: Box::new(|item, user, battle, _| {
				let armor = rand::thread_rng().gen_range(5..=10);
				user.add_armor(armor);
				battle.log.add(Entry::Item(item.icon.clone(), format!("{} equipped a shield, gaining {} armor.", user.name(), armor)));
			}),
		},
	]
}

fn create_weapons() -> Vec<Weapon> {
	vec![
		Weapon {
			name: "Dagger".to_string(),
			icon: '🗡'.into(),
			damage_range: 10..=15,
			crit_ratio: (5, 100),
			crit_multiplier: 3,
			..Default::default()
		},
		Weapon {
			name: "Hammer".to_string(),
			icon: '🔨'.into(),
			damage_range: 15..=30,
			..Default::default()
		},
		Weapon {
			name: "Spear".to_string(),
			icon: '⚔'.into(),
			pierce: 5,
			..Default::default()
		},
		Weapon {
			name: "Sword".to_string(),
			icon: '⚔'.into(),
			crit_ratio: (7, 100),
			..Default::default()
		},
	]
}

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
	pub fn new(user: User, ctx: Context<'a>, is_p1: bool) -> Self {
		let items = {
			create_items()
				 .into_iter()
				.choose_multiple(&mut rand::thread_rng(), 3)
				.into_iter()
				.fold(HashMap::new(), |mut acc, item| {
					acc.insert(item.id, item);
					acc
				})
		};

		let weapon = {
			create_weapons()
				 .into_iter()
				.choose(&mut rand::thread_rng())
				.unwrap()
		};

		Self {
			user,
			id: Uuid::new_v4(),
			is_p1,
			ctx,
			health: 100,
			max_health: 100,
			weapon,
			items,
			armor: 0,
		}
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
				battle.message.edit(self.ctx.discord(), |m|
					m.embed(|e| create_battle_embed(e, &self_display, &opponent_display, battle.p1_turn, &battle.log))
						.components(|c| create_battle_components(c, false, self.items.is_empty()))
				).await?;
			} else {
				battle.message.edit(self.ctx.discord(), |m|
					m.embed(|e| create_battle_embed(e, &opponent_display, &self_display, battle.p1_turn, &battle.log))
						.components(|c| create_battle_components(c, false, self.items.is_empty()))
				).await?;
			}

			let interaction = battle.message
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

						battle.message.edit(self.ctx.discord(), |m|
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

		let message = self.ctx.send(|m|
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
		).await?.message().await?;

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

					item.use_item(self, battle, opponent);
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