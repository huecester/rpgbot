use poise::serenity_prelude as serenity;

pub struct Player {
	user: serenity::User,
}

pub struct Battle {
	p1: Player,
	p2: Player,
}
