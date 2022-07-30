use std::ops::Bound;

#[derive(Queryable)]
pub struct QueryItem {
	pub id: i32,
	pub name: String,
	pub description: String,
	pub icon: String,
	pub lua: String,
}

#[derive(Queryable)]
pub struct QueryWeapon {
	pub id: i32,
	pub name: String,
	pub icon: String,
	pub damage_range: Option<(Bound<i32>, Bound<i32>)>,
	pub crit_ratio: Option<f64>,
	pub crit_multiplier: Option<i32>,
	pub pierce: Option<i32>,
}