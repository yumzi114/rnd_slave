
use tokio;
use mini_redis::{client, Result};

#[cfg(unix)]
const SERIAL_DEVICE: &'static str = env!("SERIAL_DEVICE");


#[tokio::main]
async fn main() -> Result<()> {

    println!("Hello, world!");
    Ok(())
}
