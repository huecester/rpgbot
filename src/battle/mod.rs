mod util;
mod log;
mod player;

use crate::{types::*, util::base_embed};
use util::*;
use log::{Log, LogEntry};
use player::Player;

use poise::serenity_prelude::{Message, User};
use rand::Rng;

pub struct Battle<'a> {
	ctx: Context<'a>,
	reply: Option<Message>,
	p1: Player,
	p2: Player,
	p1_turn: bool,
	log: Log,
}

impl<'a> Battle<'a> {
	pub fn new(ctx: Context<'a>, p1: User, p2: User) -> Self {
		Battle {
			ctx,
			reply: None,
			p1: p1.into(),
			p2: p2.into(),
			p1_turn: rand::random(),
			log: Log::new(),
		}
	}

	pub async fn start(&mut self) -> Result<(), Error> {
		if self.send_invite().await? {
			self.battle_loop().await?;
			Ok(())
		} else {
			let reply = self.reply.as_mut().unwrap();
			reply.edit(self.ctx.discord(), |m| m.components(|c| c)).await?;
			reply.reply(self.ctx.discord(), format!("{} ran away.", self.p2.mention())).await?;
			Ok(())
		}
	}

	async fn send_invite(&mut self) -> Result<bool, Error> {
		self.reply = Some(self.ctx.send(|m|
			m.embed(|e| {
					let e = base_embed(e)
						.title("⚔ Duel Invitation")
						.description(format!("{} challenged {} to a duel!", self.p1.mention(), self.p2.mention()))
						.fields(vec![
							(&self.p1.user().name, &self.p1, true),
							(&self.p2.user().name, &self.p2, true),
						]);

					if let Some(url) = self.p1.user().avatar_url() {
						e.thumbnail(url)
					} else {
						e
					}
				}
			).components(|c| create_invite_action_row(c, false))
		).await?.message().await?);

		let interaction = self.reply
			.as_ref()
			.unwrap()
			.await_component_interaction(self.ctx.discord())
			.author_id(self.p2.user().id)
			.await;

		if let Some(m) = interaction {
			m.defer(self.ctx.discord()).await?;

			match &*m.data.custom_id {
				"fight" => Ok(true),
				"run" => Ok(false),
				other => Err(format!("Unknown button ID {other}.").into()),
			}
		} else {
			self.ctx.say("The invitation timed out.").await?;
			Ok(false)
		}
	}

	async fn battle_loop(&mut self) -> Result<(), Error> {
		while self.p1.health() > 0 && self.p2.health() > 0 {
			let current_player = if self.p1_turn { &self.p1 } else { &self.p2 };
			let reply = self.reply.as_mut().unwrap();
			reply.edit(self.ctx.discord(), |m|
				m.embed(|e| create_battle_embed(e, &self.p1, &self.p2, self.p1_turn, &self.log))
					.components(|c| create_battle_components(c))
			).await?;

			let interaction = reply
				.await_component_interaction(self.ctx.discord())
				.author_id(current_player.user().id)
				.await;

			if let Some(m) = interaction {
				m.defer(self.ctx.discord()).await?;

				let mut rand = rand::thread_rng();
				match &*m.data.custom_id {
					"attack" => {
						let current_opponent = if self.p1_turn { &self.p2 } else { &self.p1 };

						let mut damage: usize = rand.gen_range(1..=25);
						let critical = rand.gen_bool(1.0 / 100.0);

						if critical {
							damage = damage.checked_mul(2).unwrap_or(usize::MAX);
							self.log.add(LogEntry::Critical(current_player.user().name.clone(), current_opponent.user().name.clone(), damage));
						} else {
							self.log.add(LogEntry::Attack(current_player.user().name.clone(), current_opponent.user().name.clone(), damage));
						}

						if self.p1_turn {
							self.p2.damage(damage);
						} else {
							self.p1.damage(damage);
						}
					},
					"surrender" => {
						self.log.add(LogEntry::Surrender(current_player.user().name.clone()));
						if self.p1_turn {
							self.p1.set_health(0);
						} else {
							self.p2.set_health(0);
						}
					},
					other => return Err(format!("Unknown button ID {other}.").into()),
				}
			} else {
				return Err("There was an error during the battle.".into());
			}

			self.p1_turn = !self.p1_turn;
		}

		let winner = {
			if self.p1.health() > 0 && self.p2.health() == 0 {
				Some(&self.p1)
			} else if self.p2.health() > 0 && self.p1.health() == 0 {
				Some(&self.p2)
			} else {
				None
			}
		};

		self.reply
			.as_mut()
			.unwrap()
			.edit(self.ctx.discord(), |m|
				if let Some(winner) = winner {
					m.embed(|e| {
						let e = base_embed(e)
							.title(format!("🏆 {} won!", winner.user().name))
							.field("Log", &self.log, false);

						if let Some(url) = winner.user().avatar_url() {
							e.thumbnail(url)
						} else {
							e
						}
					}).components(|c| c)
				} else {
					m.embed(|e| base_embed(e)
						.title("The battle was a tie...")
						.field("Log", &self.log, false)
					).components(|c| c)
				}
			).await?;

		Ok(())
	}
}