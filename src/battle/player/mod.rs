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
use item::{
	Item,
	Apple,
	Coin,
	FaultyWaterGun,
	Shield,
};
use util::create_battle_components;
use weapon::{
	Weapon,
	Hammer,
	Spear,
	Sword,
};

use std::{
	collections::HashMap,
	sync::{
		Weak,
		atomic::{AtomicUsize, Ordering},
	},
};
use async_trait::async_trait;
use poise::serenity_prelude::{ButtonStyle, User, UserId };
use rand::prelude::*;
use uuid::Uuid;

pub struct Player<'a> {
	user: User,
	id: Uuid,
	is_p1: bool,
	ctx: Context<'a>,
	battle: Option<Weak<Battle<'a>>>,
	health: AtomicUsize,
	max_health: usize,
	weapon: Box<dyn Weapon>,
	items: HashMap<Uuid, Box<dyn Item>>,
	armor: AtomicUsize,
}

impl<'a> Player<'a> {
	pub fn new(user: User, ctx: Context<'a>, is_p1: bool) -> Self {
		let items = {
			let all_items: Vec<Box<dyn Item>> = vec![
				Box::new(Apple::new()),
				Box::new(Apple::new()),
				Box::new(Coin::new()),
				Box::new(Coin::new()),
				Box::new(FaultyWaterGun::new()),
				Box::new(FaultyWaterGun::new()),
				Box::new(Shield::new()),
				Box::new(Shield::new()),
			];
			all_items
			 	.into_iter()
				.choose_multiple(&mut rand::thread_rng(), 3)
				.into_iter()
				.fold(HashMap::new(), |mut acc, item| {
					acc.insert(item.id().clone(), item);
					acc
				})
		};

		let weapon = {
			let all_weapons: Vec<Box<dyn Weapon>> = vec![
				Box::new(Sword::new()),
				Box::new(Spear::new()),
				Box::new(Hammer::new()),
			];
			all_weapons
			 	.into_iter()
				.choose(&mut rand::thread_rng())
				.unwrap()
		};

		Self {
			user,
			id: Uuid::new_v4(),
			is_p1,
			ctx,
			battle: None,
			health: AtomicUsize::new(100),
			max_health: 100,
			weapon,
			items,
			armor: AtomicUsize::new(0),
		}
	}

	pub fn user(&self) -> &User {
		&self.user
	}

	pub fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
	}

	async fn act(&mut self) -> Result<(), Error> {
		let battle = self.battle.as_ref().ok_or("Battle is unset.")?.upgrade().ok_or("Battle is over.")?;

		loop {
			{
				let mut message = battle.message.lock().await;
				let log = battle.log.lock().await;

				if self.is_p1 {
					let p1_display = self.info().display().await;
					let p2_display = battle.p2.lock().await.info().display().await;

					message.edit(self.ctx.discord(), |m|
						m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, battle.p1_turn.load(Ordering::Relaxed), &log))
							.components(|c| create_battle_components(c, false, self.items.is_empty()))
					).await?;
				} else {
					let p1_display = battle.p1.lock().await.info().display().await;
					let p2_display = self.info().display().await;

					message.edit(self.ctx.discord(), |m|
						m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, battle.p1_turn.load(Ordering::Relaxed), &log))
							.components(|c| create_battle_components(c, false, self.items.is_empty()))
					).await?;
				}
			}

			let interaction = battle.message.lock().await
				.await_component_interaction(self.ctx.discord())
				.author_id(self.user.id)
				.await;

			if let Some(m) = interaction {
				m.defer(self.ctx.discord()).await?;

				match &*m.data.custom_id {
					"attack" => self.weapon.attack(self, &battle, self.is_p1).await?,
					"surrender" => {
						let mut log = battle.log.lock().await;
						log.add(Entry::Surrender(self.name().clone()));
						self.set_health(0);
					},
					"item" => {
						if self.items.is_empty() {
							continue;
						}

						battle.message.lock().await.edit(self.ctx.discord(), |m|
							m.components(|c| create_battle_components(c, true, true))
						).await?;

						if !self.item().await? {
							continue;
						}
					},
					other => return Err(format!("Unknown ID {other}.").into()),
				}
			} else {
				let mut log = battle.log.lock().await;
				log.add(Entry::Timeout(self.name().clone()));
			}

			break;
		}

		Ok(())
	}

	async fn item(&mut self) -> Result<bool, Error> {
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
						.get(&item_id)
						.ok_or(format!("Item ID {} not found.", item_id))?;

					let battle = self.battle.as_ref().ok_or("Battle is unset.")?.upgrade().ok_or("Battle is over.")?;

					item.use_item(self, &battle, self.is_p1).await?;
					self.items.remove(&item_id);
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
impl<'a> Battler<'a> for Player<'a> {
	fn set_battle(&mut self, battle: Weak<Battle<'a>>) {
		self.battle = Some(battle);
	}

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

	async fn act(&mut self) -> Result<(), Error> {
		self.act().await
	}

	fn health(&self) -> usize {
		self.health.load(Ordering::Relaxed)
	}
	fn max_health(&self) -> usize {
		self.max_health
	}
	fn damage(&self, damage: usize, pierce: usize) -> usize {
		let health = self.health.load(Ordering::Relaxed);
		let armor = self.armor.load(Ordering::Relaxed);

		let damage = damage.checked_sub(armor.checked_sub(pierce).unwrap_or(0)).unwrap_or(0).min(health);
		self.health.store(health - damage, Ordering::Relaxed);
		damage
	}
	fn heal(&self, healing: usize) -> usize {
		let health = self.health.load(Ordering::Relaxed);
		let healing = healing.min(self.max_health - health);
		self.health.store(health + healing, Ordering::Relaxed);
		healing
	}

	fn set_health(&mut self, target: usize) {
		self.health.store(target.clamp(0, self.max_health), Ordering::Relaxed);
	}

	fn info(&self) -> BattlerInfo {
		BattlerInfo {
			ctx: self.ctx,
			name: self.name().clone(),
			icon: self.icon(),
			health: self.health(),
			max_health: self.max_health(),
			weapon: (self.weapon.icon(), self.weapon.name().to_string()),
			armor: self.armor.load(Ordering::Relaxed),
		}
	}
}