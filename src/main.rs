use rpgbot::start;
use std::{env, error::Error};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
	dotenv().ok();
	let token = env::var("DISCORD_TOKEN")?;

	start(token, vec![297_860_975_971_926_017_u64]).await
}
