use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::error::Error;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::time::{sleep, Duration};

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
        Err(_) => {}
    }

    Ok(())
}

async fn loop_requests() {
    let mut wait_duration = thread_rng().sample(Uniform::new(1000u64, 4000));

    while let _ = sleep(Duration::from_millis(wait_duration)).await {
        println!("spawning");
        tokio::spawn(async move {
            make_request().await;
        });

        wait_duration = thread_rng().sample(Uniform::new(1000u64, 4000));
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tokio::spawn(async {
        loop_requests().await;
    })
    .await;

    Ok(())
}
