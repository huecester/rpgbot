use std::fmt::Display;
use poise::serenity_prelude::User;

pub struct Player {
	user: User,
	health: usize,
	max_health: usize,
}

impl Player {
	pub fn user(&self) -> &User {
		&self.user
	}

	pub const fn health(&self) -> usize {
		self.health
	}

	pub fn mention(&self) -> String {
		format!("<@{}>", self.user.id)
	}

	pub fn damage(&mut self, damage: usize) {
		self.health -= damage.min(self.health);
	}

	pub fn set_health(&mut self, target: usize) {
		self.health = target.clamp(0, self.max_health);
	}
}

impl Display for Player {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "❤ {}/{}", self.health, self.max_health)
	}
}

impl From<User> for Player {
	fn from(user: User) -> Self {
		Self {
			user,
			health: 100,
			max_health: 100,
		}
	}
}