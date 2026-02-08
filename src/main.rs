use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let api_key = std::env::var("BINANCE_API_KEY")?;
    let api_secret = std::env::var("BINANCE_API_SECRET")?;

    println!("Credenciales cargadas correctamente");

    Ok(())
}
