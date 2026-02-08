use std::error::Error;

mod stream;
mod state;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    stream::user_stream::spawn_user_stream().await?;

    Ok(())
}
