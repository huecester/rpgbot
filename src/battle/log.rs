use std::fmt::Display;

#[non_exhaustive]
pub enum Entry {
	Attack(String, String, usize),
	Critical(String, String, usize),
	Surrender(String),
}

impl Display for Entry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let entry = match self {
			Entry::Attack(p1, p2, damage) => format!("⚔ {p1} attacked {p2} for {damage} damage."),
			Entry::Critical(p1, p2, damage) => format!("💥 {p1} got a critical hit on {p2} for {damage} damage!"),
			Entry::Surrender(player) => format!("🏳 {player} surrendered."),
		};

		write!(f, "{}", entry)
	}
}

pub struct Log(Vec<Entry>);

impl Log {
	pub const fn new() -> Self {
		Self(vec![])
	}

	pub fn add(&mut self, entry: Entry) {
		self.0.push(entry);
	}

	pub fn get_last_entries(&self, n: usize) -> Option<Vec<&Entry>> {
		if self.0.is_empty() {
			None
		} else {
			Some(self.0.iter().rev().take(n).collect())
		}
	}
}

impl Display for Log {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		const MAX_ENTRIES: usize = 30;

		let log = self.0.iter().rev().take(MAX_ENTRIES).fold(String::new(), |acc, entry| format!("{}\n{}", acc, entry));
		write!(f, "{}", log)
	}
}