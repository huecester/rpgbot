use crate::{
	prelude::*,
	battle::{
		Battle,
		Battler,
		log::Entry,
	},
};
use super::{item::Item, Player};

use async_trait::async_trait;
use poise::serenity_prelude::ReactionType;
use rand::Rng;
use uuid::Uuid;
