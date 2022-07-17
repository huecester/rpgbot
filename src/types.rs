use std::{collections::HashMap, error, sync::RwLock};
use poise::serenity_prelude::{User, UserId};
use uuid::Uuid;

pub type Error = Box<dyn error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

#[derive(Debug, Default)]
pub struct Data {
	pub battles: RwLock<HashMap<Uuid, Vec<Option<UserId>>>>,
}

impl Data {
	pub fn check_for_user_in_battle(&self, user: &User) -> bool {
		self.battles.read().unwrap().values().flatten().any(|id| id.as_ref() == Some(&user.id))
	}
}