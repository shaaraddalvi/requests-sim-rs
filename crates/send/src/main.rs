use tokio::net::TcpStream;
use tokio::time::{sleep, Duration};
use tokio::prelude::*;
use std::error::Error;
use rand::{thread_rng, Rng};
use rand::distributions::Uniform;

async fn make_request() -> Result<(), Box<dyn Error>> {
    // Connect to a peer
    let mut stream = TcpStream::connect("127.0.0.1:8900").await?;

    // Write some data.
    // stream.write_all(b"hello world!").await?;
    stream.write_u8(12).await?;
    let mut buffer = String::new();
    match stream.read_to_string(&mut buffer).await {
        Ok(_) => {
            println!("{}", buffer);
        }
        Err(_) => {

        }
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {

    let mut wait_duration = thread_rng().sample(Uniform::new(1000u64, 4000));

    while let _ = sleep(Duration::from_millis(wait_duration)).await {
        println!("spawning");
        tokio::spawn(async move  {
            make_request().await;
        });

        wait_duration = thread_rng().sample(Uniform::new(1000u64, 4000));
    }

    Ok(())
}