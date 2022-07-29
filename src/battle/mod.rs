mod log;
mod player;
mod util;

pub use player::Player;

use crate::{prelude::*, util::base_embed};
use log::Log;
use player::create_battle_embed;
use util::{BattlerInfo, create_invite_action_row};

use async_trait::async_trait;
use poise::{
	ReplyHandle,
	serenity_prelude::{User, UserId},
};
use uuid::Uuid;

#[async_trait]
pub trait Battler: Send + Sync {
	fn user_id(&self) -> Option<UserId> { None }
	fn id(&self) -> &Uuid;
	fn name(&self) -> &String;
	fn icon(&self) -> Option<String> { None }

	async fn act(&mut self, battle: &mut Battle, opponent: &mut dyn Battler) -> Result<(), Error>;

	fn health(&self) -> usize;
	fn max_health(&self) -> usize;
	fn armor(&self) -> usize;

	fn set_health(&mut self, health: usize);
	fn set_armor(&mut self, armor: usize);

	fn info(&self) -> BattlerInfo;
}

impl<'a> dyn Battler + 'a {
	fn damage(&mut self, damage: usize, pierce: usize) -> usize {
		let damage = damage.saturating_sub(self.armor().saturating_sub(pierce)).min(self.health());
		self.set_health(self.health() - damage);
		damage
	}
	fn heal(&mut self, healing: usize) -> usize {
		let healing = healing.min(self.max_health() - self.health());
		self.set_health(self.health() + healing);
		healing
	}

	fn add_armor(&mut self, armor: usize) {
		self.set_armor(self.armor().saturating_add(armor));
	}
}

pub struct Battle<'a> {
	id: Uuid,
	ctx: Context<'a>,
	reply: ReplyHandle<'a>,
	p1_turn: bool,
	log: Log,
}

impl<'a> Battle<'a> {
	fn new(ctx: Context<'a>, reply: ReplyHandle<'a>) -> Self {
		Self {
			id: Uuid::new_v4(),
			ctx,
			reply,
			p1_turn: rand::random(),
			log: Log::new(),
		}
	}

	pub async fn send_invite(ctx: Context<'a>, u1: User, u2: User) -> Result<(), Error> {
		let mut p1 = Player::new(u1, ctx, true);
		let mut p2 = Player::new(u2, ctx, false);

		let p1_display = p1.info().display().await;
		let p2_display = p2.info().display().await;

		let reply = ctx.send(|m|
			m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, true, &Log::new())
				.title("⚔ Duel Invitation")
				.description(format!("{} challenged {} to a duel!", &p1.mention(), &p2.mention()))
			).components(|c| create_invite_action_row(c, false))
		).await?;

		let interaction = reply
			.message()
			.await?
			.await_component_interaction(ctx.discord())
			.author_id(p2.user().id)
			.await;

		if let Some(m) = interaction {
			m.defer(ctx.discord()).await?;

			match &*m.data.custom_id {
				"fight" => {
					if ctx.data().check_for_user_in_battle(p2.user()) {
						reply.edit(ctx, |m| m.components(|c| c)).await?;
						ctx.send(|c| c.content("You cannot be in two battles at once.").ephemeral(true)).await?;
						return Ok(());
					}
					let mut battle = Battle::new(ctx, reply);
					battle.start(&mut p1 as &mut dyn Battler, &mut p2 as &mut dyn Battler).await
				}
				"run" => {
					reply.edit(ctx, |m| m.components(|c| c)).await?;
					reply.message().await?.reply(ctx.discord(), format!("{} ran away.", p2.mention())).await?;
					Ok(())
				},
				other => Err(format!("Unknown button ID {other}.").into()),
			}
		} else {
			ctx.say("The invitation timed out.").await?;
			Ok(())
		}
	}

	async fn start(&mut self, p1: &mut dyn Battler, p2: &mut dyn Battler) -> Result<(), Error> {
		let (p1_id, p2_id) = (p1.user_id(), p2.user_id());
		self.ctx.data().battles.write().unwrap().insert(self.id, vec![p1_id, p2_id]);
		self.battle_loop(p1, p2).await?;
		Ok(())
	}

	async fn battle_loop(&mut self, p1: &mut dyn Battler, p2: &mut dyn Battler) -> Result<(), Error> {
		while p1.health() > 0 && p2.health() > 0 {
			if self.p1_turn {
				p1.act(self, p2).await?;
			} else {
				p2.act(self, p1).await?;
			};

			self.p1_turn = !self.p1_turn;
		}

		let winner = p1.health() > 0 && p2.health() == 0 || p2.health() > 0 && p1.health() == 0;
		let p1_win = winner && p2.health() == 0;

		self.reply.edit(self.ctx, |m|
			if winner {
				m.embed(|e| {
					let e = base_embed(e)
						.field("Log", &self.log, false);

					if p1_win {
						let e = e.title(format!("🏆 {} won!", p1.name()));
						if let Some(url) = p1.icon() {
							e.thumbnail(url)
						} else {
							e
						}
					} else {
						let e = e.title(format!("🏆 {} won!", p2.name()));
						if let Some(url) = p2.icon() {
							e.thumbnail(url)
						} else {
							e
						}
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

impl Drop for Battle<'_> {
	fn drop(&mut self) {
		self.ctx.data().battles.write().unwrap().remove(&self.id);
	}
}
