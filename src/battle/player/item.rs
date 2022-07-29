use crate::battle::{Battle, Battler};

use poise::serenity_prelude::{CreateSelectMenuOption, ReactionType};
use uuid::Uuid;

type ItemCallback = Box<dyn Fn(&Item, &mut dyn Battler, &mut Battle, &mut dyn Battler) + Send + Sync>;
pub struct Item {
    pub name: String,
    pub id: Uuid,
    pub description: String,
    pub icon: ReactionType,
    pub cb: ItemCallback,
}

impl Item {
    pub fn use_item(&self, user: &mut dyn Battler, battle: &mut Battle, opponent: &mut dyn Battler) {
        (self.cb)(self, user, battle, opponent);
    }

    pub fn as_option<'a>(&self, o: &'a mut CreateSelectMenuOption) -> &'a mut CreateSelectMenuOption {
        o.label(&self.name)
            .value(&self.id)
            .description(&self.description)
            .emoji(self.icon.clone())
    }
}
