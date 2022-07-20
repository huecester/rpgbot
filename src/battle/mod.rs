mod log;
mod player;
mod util;

pub use player::Player;
use crate::{prelude::*, util::base_embed};
use log::Log;
use util::{BattlerInfo, create_battle_components, create_battle_embed, create_invite_action_row};

use std::sync::{
	Arc,
	Weak,
	atomic::{AtomicBool, Ordering},
};
use async_trait::async_trait;
use poise::serenity_prelude::{Message, Mutex, User, UserId};
use uuid::Uuid;

#[async_trait]
pub trait Battler<'a>: Send + Sync {
	fn set_battle(&mut self, battle: Weak<Battle<'a>>);

	fn user_id(&self) -> Option<UserId> { None }
	fn id(&self) -> &Uuid;
	fn name(&self) -> &String;
	fn icon(&self) -> Option<String> { None }

	async fn act(&mut self) -> Result<(), Error>;

	fn health(&self) -> usize;
	fn max_health(&self) -> usize;
	fn damage(&mut self, damage: usize);

	fn set_health(&mut self, target: usize);

	fn info(&self) -> BattlerInfo;
}

type SharedBattler<T> = Arc<Mutex<T>>;
pub struct Battle<'a> {
	id: Uuid,
	ctx: Context<'a>,
	message: Mutex<Message>,
	p1: SharedBattler<dyn Battler<'a> + 'a>,
	p2: SharedBattler<dyn Battler<'a> + 'a>,
	p1_turn: AtomicBool,
	log: Mutex<Log>,
}

impl<'a> Battle<'a> {
	async fn new<T, U>(ctx: Context<'a>, message: Message, p1: T, p2: U) -> Arc<Battle<'a>>
		where
			T: 'a + Battler<'a>,
			U: 'a + Battler<'a>,
	{
		let battle = {
			let p1 = Arc::new(Mutex::new(p1));
			let p2 = Arc::new(Mutex::new(p2));

			Self {
				id: Uuid::new_v4(),
				ctx,
				message: Mutex::new(message),
				p1,
				p2,
				p1_turn: AtomicBool::new(rand::random()),
				log: Mutex::new(Log::new()),
			}
		};

		let pointer = Arc::new(battle);
		pointer.p1.lock().await.set_battle(Arc::downgrade(&pointer));
		pointer.p2.lock().await.set_battle(Arc::downgrade(&pointer));

		pointer
	}

	pub async fn send_invite(ctx: Context<'a>, u1: User, u2: User) -> Result<(), Error> {
		let p1 = Player::new(u1, ctx, true);
		let p2 = Player::new(u2, ctx, false);

		let p1_display = p1.info().display().await;
		let p2_display = p2.info().display().await;

		let mut message = ctx.send(|m|
			m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, true, &Log::new())
				.title("⚔ Duel Invitation")
				.description(format!("{} challenged {} to a duel!", &p1.mention(), &p2.mention()))
			).components(|c| create_invite_action_row(c, false))
		).await?.message().await?;

		let interaction = message
			.await_component_interaction(ctx.discord())
			.author_id(p2.user().id)
			.await;

		if let Some(m) = interaction {
			m.defer(ctx.discord()).await?;

			match &*m.data.custom_id {
				"fight" => {
					if ctx.data().check_for_user_in_battle(p2.user()) {
						message.edit(ctx.discord(), |m| m.components(|c| c)).await?;
						ctx.send(|c| c.content("You cannot be in two battles at once.").ephemeral(true)).await?;
						return Ok(());
					}
					let battle = Battle::new(ctx, message, p1, p2).await;
					battle.start().await
				}
				"run" => {
					message.edit(ctx.discord(), |m| m.components(|c| c)).await?;
					message.reply(ctx.discord(), format!("{} ran away.", p2.mention())).await?;
					Ok(())
				},
				other => Err(format!("Unknown button ID {other}.").into()),
			}
		} else {
			ctx.say("The invitation timed out.").await?;
			Ok(())
		}
	}

	async fn start(&self) -> Result<(), Error> {
		let (p1_id, p2_id) = (self.p1.lock().await.user_id(), self.p2.lock().await.user_id());
		self.ctx.data().battles.write().unwrap().insert(self.id, vec![p1_id, p2_id]);
		self.battle_loop().await?;
		Ok(())
	}

	async fn battle_loop(&self) -> Result<(), Error> {
		while self.p1.lock().await.health() > 0 && self.p2.lock().await.health() > 0 {
			let p1_turn = self.p1_turn.load(Ordering::Relaxed);

			{
				let p1 = self.p1.lock().await;
				let p2 = self.p2.lock().await;

				let p1_display = p1.info().display().await;
				let p2_display = p2.info().display().await;
				let log = self.log.lock().await;

				self.message.lock().await.edit(self.ctx.discord(), |m|
					m.embed(|e| create_battle_embed(e, &p1_display, &p2_display, p1_turn, &log))
						.components(|c| create_battle_components(c))
				).await?;
			}

			if p1_turn {
				self.p1.lock().await.act().await?;
			} else {
				self.p2.lock().await.act().await?;
			};

			self.p1_turn.store(!p1_turn, Ordering::Relaxed);
		}

		let (winner, p1_win) = {
			let p1 = self.p1.lock().await;
			let p2 = self.p2.lock().await;

			let winner = p1.health() > 0 && p2.health() == 0 || p2.health() > 0 && p1.health() == 0;
			let p1_win = winner && p2.health() == 0;

			(winner, p1_win)
		};

		let (p1_name, p1_icon) = {
			let p1 = self.p1.lock().await;
			(p1.name().clone(), p1.icon())
		};
		let (p2_name, p2_icon) = {
			let p2 = self.p2.lock().await;
			(p2.name().clone(), p2.icon())
		};
		let log = self.log.lock().await.clone();

		self.message.lock().await.edit(self.ctx.discord(), |m|
			if winner {
				m.embed(|e| {
					let e = base_embed(e)
						.field("Log", log, false);

					if p1_win {
						let e = e.title(format!("🏆 {} won!", p1_name));
						if let Some(url) = p1_icon {
							e.thumbnail(url)
						} else {
							e
						}
					} else {
						let e = e.title(format!("🏆 {} won!", p2_name));
						if let Some(url) = p2_icon {
							e.thumbnail(url)
						} else {
							e
						}
					}

				}).components(|c| c)
			} else {
				m.embed(|e| base_embed(e)
					.title("The battle was a tie...")
					.field("Log", log, false)
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
