use std::fmt::Display;

#[non_exhaustive]
pub enum LogEntry {
	Attack(String, String, usize),
	Critical(String, String, usize),
	Surrender(String),
}

impl Display for LogEntry {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let entry = match self {
			LogEntry::Attack(p1, p2, damage) => format!("⚔ {p1} attacked {p2} for {damage} damage."),
			LogEntry::Critical(p1, p2, damage) => format!("💥 {p1} got a critical hit on {p2} for {damage} damage!"),
			LogEntry::Surrender(player) => format!("🏳 {player} surrendered."),
		};

		write!(f, "{}", entry)
	}
}

pub struct Log(Vec<LogEntry>);

impl Log {
	pub fn new() -> Self {
		Log(vec![])
	}

	pub fn add(&mut self, entry: LogEntry) {
		self.0.push(entry);
	}

	pub fn get_last_entries(&self, n: usize) -> Option<Vec<&LogEntry>> {
		if self.0.len() > 0 {
			Some(self.0.iter().rev().take(n).collect())
		} else {
			None
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