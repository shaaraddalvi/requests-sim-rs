use tokio::net::TcpStream;
use tokio::prelude::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
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