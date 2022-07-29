use poise::serenity_prelude::CreateEmbed;

pub fn base_embed(e: &mut CreateEmbed) -> &mut CreateEmbed {
    e.color((0x51, 0x68, 0xf2))
}