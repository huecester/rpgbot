use std::ops::RangeInclusive;

#[derive(Queryable)]
pub struct Item {
	pub id: usize,
	pub name: String,
	pub description: String,
	pub icon: char,
	pub lua: String,
}

#[derive(Queryable)]
pub struct Weapon {
	pub id: usize,
	pub name: String,
	pub icon: char,
	pub damage_range: Option<RangeInclusive<usize>>,
	pub crit_ratio: Option<f64>,
	pub crit_multiplier: Option<usize>,
	pub pierce: Option<usize>,
}