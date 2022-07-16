use crate::{ types::*, util::base_embed };

use std::fmt::Display;
use poise::serenity_prelude::{self as serenity, ButtonStyle, CreateComponents};

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

struct Player {
	user: serenity::User,
	health: usize,
	max_health: usize,
}

impl Player {
	fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
	}
}

impl Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "❤ {}/{}", self.health, self.max_health)
	}
}

impl From<serenity::User> for Player {
	fn from(user: serenity::User) -> Self {
		Player {
			user,
			health: 100,
			max_health: 100,
		}
	}
}

pub struct Battle<'a> {
	ctx: Context<'a>,
	reply: Option<serenity::Message>,
	p1: Player,
	p2: Player,
}

impl<'a> Battle<'a> {
	pub fn new(ctx: Context<'a>, p1: serenity::User, p2: serenity::User) -> Self {
		Battle {
			ctx,
			reply: None,
			p1: p1.into(),
			p2: p2.into(),
		}
	}

	pub async fn start(&mut self) -> Result<(), Error> {
		if self.send_invite().await? {
			self.reply
				.as_mut()
				.unwrap()
				.edit(self.ctx.discord(), |m|
					m.content("Ok")
						.components(|c| c)
				).await?;
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

		self.reply
			.as_mut()
			.unwrap()
			.edit(self.ctx.discord(), |m| m.components(|c| create_invite_action_row(c, true))).await?;

		if let Some(m) = interaction {
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
}