use tokio::{net::TcpListener, stream::StreamExt};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut listener = TcpListener::bind("127.0.0.1:8900").await.unwrap();
    while let Some(stream) = listener.next().await {
        match stream {
            Ok(mut stream) => {
                println!("new client!");
                // let mut buffer = String::new();
                let buf = stream.read_u8().await;
                // let msg = stream.read_u8(&mut buf).await;
                match buf {
                    Ok(buf) => {
                        println!("{:?}", buf);
                        sleep(Duration::from_millis(2000)).await;
                        stream.write_all("Got".as_bytes()).await;
                    }
                    Err(_) => {

                    }
                }
            }
            Err(_) => { /* connection failed */ }
        }
    }
}