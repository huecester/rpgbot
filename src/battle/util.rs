use crate::util::base_embed;
use super::log::Log;

use poise::serenity_prelude::{ButtonStyle, CreateComponents, CreateEmbed};

pub struct BattlerInfo {
	pub name: String,
	pub icon: Option<String>,
	pub stats: String,
}

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

pub fn create_battle_embed<'a, 'b>(e: &'a mut CreateEmbed, p1: BattlerInfo, p2: BattlerInfo, p1_turn: bool, log: Log) -> &'a mut CreateEmbed {
	let (current_name, current_icon) = {
		if p1_turn {
			(&p1.name, p1.icon)
		} else {
			(&p2.name, p2.icon)
		}
	};

	let log = log.get_last_entries(3).map_or_else(|| "---".to_string(), |log| log.iter().fold(String::new(), |acc, entry| format!("{}\n{}", acc, entry)));

	let e = base_embed(e)
		.title(format!("{}'s turn", current_name))
		.fields(vec![
			(p1.name, p1.stats, true),
			(p2.name, p2.stats, true),
		])
		.field("Log", log, false);

	if let Some(url) = current_icon {
		e.thumbnail(url)
	} else {
		e
	}
}

pub fn create_battle_components(c: &mut CreateComponents) -> &mut CreateComponents {
	c.create_action_row(|r|
		r.create_button(|b|
			b.custom_id("attack")
				.emoji('⚔')
				.label("Attack")
				.style(ButtonStyle::Primary)
		).create_button(|b|
			b.custom_id("surrender")
				.emoji('🏳')
				.label("Surrender")
				.style(ButtonStyle::Danger)
		)
	)
}