mod item;

use crate::prelude::*;
use item::Item;
use super::{
	Battle,
	Battler,
	log::Entry,
	util::{BattlerInfo, create_battle_components, create_battle_embed},
};

use std::sync::{
	Weak,
	atomic::Ordering,
};
use async_trait::async_trait;
use poise::serenity_prelude::{User, UserId};
use rand::Rng;
use uuid::Uuid;

pub struct Player<'a> {
	user: User,
	id: Uuid,
	is_p1: bool,
	ctx: Context<'a>,
	battle: Option<Weak<Battle<'a>>>,
	health: usize,
	max_health: usize,
	items: Vec<Box<dyn Item>>,
}

impl<'a> Player<'a> {
	pub fn new(user: User, ctx: Context<'a>, is_p1: bool) -> Self {
		Self {
			user,
			id: Uuid::new_v4(),
			is_p1,
			ctx,
			battle: None,
			health: 100,
			max_health: 100,
			items: vec![],
		}
	}

	pub fn user(&self) -> &User {
		&self.user
	}

	pub fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
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
		let battle = self.battle.as_ref().unwrap().upgrade().unwrap();
		let mut message = battle.message.lock().await;
		let mut log = battle.log.lock().await;

		{
			if self.is_p1 {
				let p1_display = self.info().display().await;
				let p2_display = battle.p2.lock().await.info().display().await;

				message.edit(self.ctx.discord(), |m|
					m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, battle.p1_turn.load(Ordering::Relaxed), &log))
						.components(|c| create_battle_components(c))
				).await?;
			} else {
				let p1_display = battle.p1.lock().await.info().display().await;
				let p2_display = self.info().display().await;

				message.edit(self.ctx.discord(), |m|
					m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, battle.p1_turn.load(Ordering::Relaxed), &log))
						.components(|c| create_battle_components(c))
				).await?;
			}
		}

		let interaction = message
			.await_component_interaction(self.ctx.discord())
			.author_id(self.user.id)
			.await;

		if let Some(m) = interaction {
			m.defer(self.ctx.discord()).await?;

			match &*m.data.custom_id {
				"attack" => {
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
				},
				"surrender" => {
					log.add(Entry::Surrender(self.name().clone()));
					self.set_health(0);
				},
				other => return Err(format!("Unknown button ID {other}.").into()),
			}
		} else {
			log.add(Entry::Timeout(self.name().clone()));
			self.set_health(0);
		}

		Ok(())
	}

	fn health(&self) -> usize {
		self.health
	}
	fn max_health(&self) -> usize {
		self.max_health
	}
	fn damage(&mut self, damage: usize) {
		self.health -= damage.min(self.health);
	}

	fn set_health(&mut self, target: usize) {
		self.health = target.clamp(0, self.max_health);
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