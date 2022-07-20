use crate::{
	battle::{
		log::Log,
		util::BattlerDisplay,
	},
	util::base_embed,
};

use poise::serenity_prelude::{CreateEmbed, CreateComponents, ButtonStyle};

pub fn create_battle_embed<'a>(e: &'a mut CreateEmbed, p1: &BattlerDisplay, p2: &BattlerDisplay, p1_turn: bool, log: &Log) -> &'a mut CreateEmbed {
	let current_player = if p1_turn { p1 } else { p2 };

	let log = log.get_last_entries(3).map_or_else(|| "---".to_string(), |log| log.iter().fold(String::new(), |acc, entry| format!("{}\n{}", acc, entry)));

	let e = base_embed(e)
		.title(format!("{}'s turn", &current_player.0))
		.fields(vec![
			(&p1.0, &p1.2, true),
			(&p2.0, &p2.2, true),
		])
		.field("Log", log, false);

	if let Some(url) = &current_player.1 {
		e.thumbnail(url)
	} else {
		e
	}
}

pub fn create_battle_components(c: &mut CreateComponents, disabled: bool, disable_items: bool) -> &mut CreateComponents {
	c.create_action_row(|r|
		r.create_button(|b|
			b.custom_id("attack")
				.emoji('⚔')
				.label("Attack")
				.style(ButtonStyle::Primary)
				.disabled(disabled)
		).create_button( |b|
			b.custom_id("item")
				.emoji('🎒')
				.label("Items...")
				.style(ButtonStyle::Primary)
				.disabled(disabled || disable_items)
		).create_button(|b|
			b.custom_id("surrender")
				.emoji('🏳')
				.label("Surrender")
				.style(ButtonStyle::Danger)
				.disabled(disabled)
		)
	)
}