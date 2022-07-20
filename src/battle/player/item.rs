use poise::serenity_prelude::Emoji;

pub trait Item: Send + Sync {
	fn name(&self) -> &str;
	fn description(&self) -> &str;
	fn icon(&self) -> &Emoji;
	fn use_item(&mut self);
}

impl dyn Item {
	fn field(&self) -> (String, &str, bool) {
		(format!("{} {}", self.icon(), self.name()), self.description(), true)
	}
}