use rpgbot::start;
use std::{
    env,
    error::Error,
};
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv().unwrap();
    let token = env::var("DISCORD_TOKEN")?;

    start(token, vec![297860975971926017u64]).await
}
