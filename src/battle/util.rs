use crate::prelude::*;

use poise::serenity_prelude::{ButtonStyle, CreateComponents, Emoji, Guild, ReactionType, read_image};

async fn get_or_create_emoji(emojis: &[Emoji], name: &str, guild: &Guild, ctx: Context<'_>) -> Result<Emoji, Error> {
	if let Some(emoji) = emojis.iter().find(|emoji| emoji.name == name) {
		Ok(emoji.clone())
	} else {
		Ok(guild.create_emoji(ctx.discord(), name, &read_image(format!("./img/{}.png", name))?).await?)
	}
}

async fn create_health_bar(ctx: Context<'_>, health: usize, max_health: usize) -> Result<Vec<Emoji>, Error> {
	let guild = ctx.guild().ok_or("No guild found.")?;
	let emojis = guild.emojis(ctx.discord()).await?;

	const HEALTHBAR_LENGTH: usize = 6;
	let percent_health_remaining = health as f64 / max_health as f64;
	let full_bar_emojis = if percent_health_remaining == 0.0 {
		0
	} else if (percent_health_remaining - 1.0).abs() < f64::EPSILON {
		HEALTHBAR_LENGTH
	} else {
		((HEALTHBAR_LENGTH as f64 * percent_health_remaining).floor() as usize).clamp(1, HEALTHBAR_LENGTH - 1)
	};

	let mut healthbar = vec![];
	for i in 0..HEALTHBAR_LENGTH {
		let fill_type = if i < full_bar_emojis { "full" } else { "empty" };
		let bar_type = if i == 0 {
			format!("{fill_type}_start")
		} else if i == HEALTHBAR_LENGTH - 1 {
			format!("{fill_type}_end")
		} else {
			format!("{fill_type}_middle")
		};

		healthbar.push(get_or_create_emoji(&emojis, &format!("bar_{bar_type}"), &guild, ctx).await?);
	}

	if healthbar.iter().any(|emoji| !emoji.available) { return Err("Some emojis aren't available.".into()) }

	Ok(healthbar)
}

pub struct BattlerInfo<'a> {
	pub ctx: Context<'a>,
	pub name: String,
	pub icon: Option<String>,
	pub health: usize,
	pub max_health: usize,
	pub weapon: (ReactionType, String),
	pub armor: usize,
}

impl BattlerInfo<'_> {
	pub async fn display(&self) -> BattlerDisplay {
		let health = if let Ok(healthbar) = create_health_bar(self.ctx, self.health, self.max_health).await {
			let healthbar = healthbar.iter().fold(String::new(), |acc, emoji| acc + &emoji.to_string());
			format!("❤ {healthbar} {}", self.health)
		} else {
			format!("❤️ {}/{}", self.health, self.max_health)
		};

		let weapon = format!("{} {}", self.weapon.0, self.weapon.1);

		let armor = format!("🛡 {}", self.armor);


		let stats = format!("{health}\n{weapon}\n{armor}");

		BattlerDisplay(
			self.name.clone(),
			self.icon.clone(),
			stats,
		)
	}
}

pub struct BattlerDisplay(
	pub String,
	pub Option<String>,
	pub String,
);

pub fn create_invite_action_row(c: &mut CreateComponents, disabled: bool) -> &mut CreateComponents {
	c.create_action_row(|r|
		r.create_button(|b|
			b.custom_id("fight")
			 	.emoji('⚔')
				.label("Fight")
				.style(ButtonStyle::Primary)
				.disabled(disabled)
		).create_button(|b|
			b.custom_id("run")
			 	.emoji('💨')
				.label("Run")
				.style(ButtonStyle::Danger)
				.disabled(disabled)
		)
	)
}