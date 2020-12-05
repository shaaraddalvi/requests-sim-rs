use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::error::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};
use tokio::{net::TcpListener, stream::StreamExt};

async fn process_stream(mut stream: tokio::net::TcpStream) {
    let wait_duration = thread_rng().sample(Uniform::new(1000u64, 20000));

    println!("new client!");
    // let mut buffer = String::new();
    let buf = stream.read_u8().await;
    // let msg = stream.read_u8(&mut buf).await;
    match buf {
        Ok(buf) => {
            println!("{:?}", buf);

            sleep(Duration::from_millis(wait_duration)).await;
            stream.write_all("Got".as_bytes()).await;
            println!("Sent");
        }
        Err(_) => {}
    }
}

async fn handle_tcp() {
    let mut listener = TcpListener::bind("127.0.0.1:8900").await.unwrap();
    while let Some(stream) = listener.next().await {
        match stream {
            Ok(stream) => {
                tokio::spawn(async move {
                    process_stream(stream).await;
                });
            }
            Err(_) => { /* connection failed */ }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let tcp_handle = tokio::spawn(async {
        handle_tcp().await;
    });

    tcp_handle.await?;

    Ok(())
}
