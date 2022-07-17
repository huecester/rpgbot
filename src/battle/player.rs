use std::sync::Weak;

use crate::prelude::*;
use super::{Battle, Battler, log::Entry, util::BattlerInfo};

use async_trait::async_trait;
use poise::serenity_prelude::{User, UserId};
use rand::Rng;
use uuid::Uuid;

#[derive(Clone)]
pub struct Player<'a, 'b> {
	user: User,
	id: Uuid,
	is_p1: bool,
	ctx: Context<'b>,
	battle: Option<Weak<Battle<'a, 'b>>>,
	health: usize,
	max_health: usize,
}

impl<'a, 'b> Player<'a, 'b> {
	pub fn new(user: User, ctx: Context<'b>, is_p1: bool) -> Self {
		Self {
			user,
			id: Uuid::new_v4(),
			is_p1,
			ctx,
			battle: None,
			health: 100,
			max_health: 100,
		}
	}

	pub fn user(&self) -> &User {
		&self.user
	}

	pub fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
	}

	pub fn stats(&self) -> String {
		format!("❤ {}/{}", self.health(), self.max_health())
	}
}

#[async_trait]
impl<'a, 'b> Battler<'a, 'b> for Player<'a, 'b> {
	fn set_battle(&mut self, battle: Weak<Battle<'a, 'b>>) {
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

		let interaction = battle.message.lock().await
			.await_component_interaction(self.ctx.discord())
			.author_id(self.user.id)
			.await;

		let mut log = battle.log.lock().await;

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
			name: self.name().clone(),
			icon: self.icon(),
			stats: self.stats(),
		}
	}
}