mod item;
mod util;

pub use util::create_battle_embed;

use crate::prelude::*;
use self::item::Apple;

use super::{
	Battle,
	Battler,
	log::Entry,
	util::BattlerInfo,
};
use item::Item;
use util::create_battle_components;

use std::{
	collections::HashMap,
	sync::{
		Weak,
		atomic::{AtomicUsize, Ordering},
	},
};
use async_trait::async_trait;
use poise::serenity_prelude::{ButtonStyle, User, UserId };
use rand::Rng;
use uuid::Uuid;

pub struct Player<'a> {
	user: User,
	id: Uuid,
	is_p1: bool,
	ctx: Context<'a>,
	battle: Option<Weak<Battle<'a>>>,
	health: AtomicUsize,
	max_health: usize,
	items: HashMap<Uuid, Box<dyn Item>>,
}

impl<'a> Player<'a> {
	pub fn new(user: User, ctx: Context<'a>, is_p1: bool) -> Self {
		let items: HashMap<Uuid, Box<dyn Item>> = vec![
			Apple::new()
		].into_iter().fold(HashMap::new(), |mut acc, item| {
			acc.insert(item.id().clone(), Box::new(item));
			acc
		});

		Self {
			user,
			id: Uuid::new_v4(),
			is_p1,
			ctx,
			battle: None,
			health: AtomicUsize::new(100),
			max_health: 100,
			items,
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
					"attack" => self.attack().await?,
					"surrender" => {
						let mut log = battle.log.lock().await;
						log.add(Entry::Surrender(self.name().clone()));
						self.set_health(0);
					},
					"item" => {
						if self.items.is_empty() {
							self.ctx.send(|m| m.content("You have no items.").ephemeral(true)).await?;
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

	async fn attack(&self) -> Result<(), Error> {
		let battle = self.battle.as_ref().ok_or("Battle is unset.")?.upgrade().ok_or("Battle is over.")?;
		let mut log = battle.log.lock().await;

		let opponent_name = if self.is_p1 {
			battle.p2.lock().await.name().clone()
		} else {
			battle.p1.lock().await.name().clone()
		};

		let (mut damage, critical) = {
			let mut rand = rand::thread_rng();
			let damage: usize = rand.gen_range(1..=25);
			let critical = rand.gen_bool(1.0 / 100.0);
			(damage, critical)
		};

		if critical {
			damage = damage.checked_mul(2).unwrap_or(usize::MAX);
			log.add(Entry::Critical(self.name().clone(), opponent_name, damage));
		} else {
			log.add(Entry::Attack(self.name().clone(), opponent_name, damage));
		}

		if self.is_p1 {
			battle.p2.lock().await.damage(damage);
		} else {
			battle.p1.lock().await.damage(damage);
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

		if let Some(m) = interaction {
			m.defer(self.ctx.discord()).await?;
			message.delete(self.ctx.discord()).await?;

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
	fn damage(&self, damage: usize) {
		let health = self.health.load(Ordering::Relaxed);
		self.health.store(health - damage.min(health), Ordering::Relaxed);
	}
	fn heal(&self, healing: usize) {
		let health = self.health.load(Ordering::Relaxed);
		self.health.store(health + healing.min(self.max_health - health), Ordering::Relaxed);
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
		}
	}
}