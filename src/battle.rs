use crate::types::*;
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

pub struct Player {
	user: serenity::User,
}

impl From<serenity::User> for Player {
	fn from(user: serenity::User) -> Self {
		Player {
			user,
		}
	}
}

pub struct Battle<'a> {
	ctx: Context<'a>,
	p1: Player,
	p2: Player,
}

impl<'a> Battle<'a> {
	pub fn new(ctx: Context<'a>, p1: serenity::User, p2: serenity::User) -> Self {
		Battle {
			ctx,
			p1: p1.into(),
			p2: p2.into(),
		}
	}

	pub async fn start(&self) -> Result<(), Error> {
		if self.send_invite().await? {
			todo!()
		} else {
			todo!()
		}
	}

	async fn send_invite(&self) -> Result<bool, Error> {
		let mut reply = self.ctx.send(|m|
			m.components(|c| create_invite_action_row(c, false))
		).await?.message().await?;

		let interaction = reply
			.await_component_interaction(self.ctx.discord())
			.author_id(self.p2.user.id)
			.await;

		reply.edit(self.ctx.discord(), |m| m.components(|c| create_invite_action_row(c, true))).await?;

		if let Some(m) = interaction {
			match &*m.data.custom_id {
				"fight" => Ok(true),
				"run" => Ok(false),
				other => Err(format!("Invalid button ID {}.", other).into()),
			}
		} else {
			self.ctx.say("The invitation timed out.").await?;
			Ok(false)
		}
	}
}