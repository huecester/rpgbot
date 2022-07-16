use crate::{types::*, util::base_embed};

use std::fmt::Display;
use poise::serenity_prelude::{ButtonStyle, CreateComponents, CreateEmbed, Message, User};
use rand::Rng;

fn create_invite_action_row(c: &mut CreateComponents, disabled: bool) -> &mut CreateComponents {
	c.create_action_row(|r|
		r.create_button(|b|
			b.custom_id("fight")
				.label("⚔ Fight")
				.style(ButtonStyle::Primary)
				.disabled(disabled)
		).create_button(|b|
			b.custom_id("run")
				.label("💨 Run")
				.style(ButtonStyle::Danger)
				.disabled(disabled)
		)
	)
}

fn create_battle_embed<'a>(e: &'a mut CreateEmbed, p1: &Player, p2: &Player, p1_turn: bool, log: &Vec<String>) -> &'a mut CreateEmbed {
	let current_player = if p1_turn { p1 } else { p2 };

	let e = base_embed(e)
		.title(format!("{}'s turn", current_player.user.name))
		.fields(vec![
			(&p1.user.name, &p1, true),
			(&p2.user.name, &p2, true),
		])
		.field("Log", log.last().unwrap_or(&"---".to_string()), false);

	if let Some(url) = current_player.user.avatar_url() {
		e.thumbnail(url)
	} else {
		e
	}
}

fn create_battle_components<'a>(c: &'a mut CreateComponents) -> &'a mut CreateComponents {
	c.create_action_row(|r|
		r.create_button(|b|
			b.custom_id("attack")
				.label("⚔ Attack")
				.style(ButtonStyle::Primary)
		).create_button(|b|
			b.custom_id("surrender")
				.label("🏳 Surrender")
				.style(ButtonStyle::Danger)
		)
	)
}

struct Player {
	user: User,
	health: usize,
	max_health: usize,
}

impl Player {
	fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
	}

	fn damage(&mut self, damage: usize) {
		self.health -= damage.min(self.health);
	}
}

impl Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "❤ {}/{}", self.health, self.max_health)
	}
}

impl From<User> for Player {
	fn from(user: User) -> Self {
		Player {
			user,
			health: 100,
			max_health: 100,
		}
	}
}

pub struct Battle<'a> {
	ctx: Context<'a>,
	reply: Option<Message>,
	p1: Player,
	p2: Player,
	p1_turn: bool,
	log: Vec<String>,
}

impl<'a> Battle<'a> {
	pub fn new(ctx: Context<'a>, p1: User, p2: User) -> Self {
		Battle {
			ctx,
			reply: None,
			p1: p1.into(),
			p2: p2.into(),
			p1_turn: rand::random(),
			log: vec![],
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
							(&self.p1.user.name, &self.p1, true),
							(&self.p2.user.name, &self.p2, true),
						]);

					if let Some(url) = self.p1.user.avatar_url() {
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
			.author_id(self.p2.user.id)
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
		while self.p1.health > 0 && self.p2.health > 0 {
			let current_player = if self.p1_turn { &self.p1 } else { &self.p2 };
			let reply = self.reply.as_mut().unwrap();
			reply.edit(self.ctx.discord(), |m|
				m.embed(|e| create_battle_embed(e, &self.p1, &self.p2, self.p1_turn, &self.log))
					.components(|c| create_battle_components(c))
			).await?;

			let interaction = reply
				.await_component_interaction(self.ctx.discord())
				.author_id(current_player.user.id)
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
							self.log.push(format!("💥 {} got a critical hit on {} for {damage} damage!", current_player.user.name, current_opponent.user.name));
						} else {
							self.log.push(format!("⚔ {} attacked {} for {damage} damage.", current_player.user.name, current_opponent.user.name));
						}

						if self.p1_turn {
							self.p2.damage(damage);
						} else {
							self.p1.damage(damage);
						}
					},
					"surrender" => {
						self.log.push(format!("🏳 {} surrendered.", current_player.user.name));
						self.p1.health = 0;
					},	
					other => return Err(format!("Unknown button ID {other}.").into()),
				}
			} else {
				return Err("There was an error during the battle.".into());
			}

			self.p1_turn = !self.p1_turn;
		}

		let winner = {
			if self.p1.health > 0 && self.p2.health == 0 {
				Some(&self.p1)
			} else if self.p2.health > 0 && self.p1.health == 0 {
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
							.title(format!("🏆 {} won!", winner.user.name));
						
						if let Some(url) = winner.user.avatar_url() {
							e.thumbnail(url)
						} else {
							e
						}
					})
				} else {
					m.embed(|e| base_embed(e).title("The battle was a tie..."))
				}
			).await?;

		Ok(())
	}
}