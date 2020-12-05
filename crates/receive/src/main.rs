use rand::distributions::Uniform;
use rand::{thread_rng, Rng};
use std::error::Error;
use std::vec::Vec;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};
use tokio::{net::TcpListener, stream::StreamExt};
// use termion::raw::IntoRawMode;
// use std::io;
// use tui::Terminal;
// use tui::backend::TermionBackend;
// use tui::widgets::{Widget, Block, Borders};
// use tui::layout::{Layout, Constraint, Direction};

async fn process_stream(mut stream: tokio::net::TcpStream, tx: mpsc::Sender<isize>) {
    let wait_duration = thread_rng().sample(Uniform::new(1000u64, 20000));

    tx.send(1).await;
    let buf = stream.read_u8().await;

    match buf {
        Ok(_) => {
            sleep(Duration::from_millis(wait_duration)).await;
            match stream.write_all("Got".as_bytes()).await {
                Ok(_) => {
                    tx.send(-1).await;
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }
}

async fn handle_tcp(tx: mpsc::Sender<isize>) {
    let mut listener = TcpListener::bind("127.0.0.1:8900").await.unwrap();
    while let Some(stream) = listener.next().await {
        match stream {
            Ok(stream) => {
                let tx2 = tx.clone();
                tokio::spawn(async move {
                    process_stream(stream, tx2).await;
                });
            }
            Err(_) => { /* connection failed */ }
        }
    }
}

async fn displayer(mut rx: mpsc::Receiver<isize>) {
    let mut current_count: isize = 0;

    while let Some(message) = rx.recv().await {
        current_count += message;

        println!("Chrrently {} requests in flight", current_count);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<isize>(32);

    let mut handles = Vec::new();

    handles.push(tokio::spawn(async {
        handle_tcp(tx).await;
    }));

    handles.push(tokio::spawn(async {
        displayer(rx).await;
    }));

    // let stdout = io::stdout().into_raw_mode()?;
    // let backend = TermionBackend::new(stdout);
    // let mut terminal = Terminal::new(backend)?;

    // terminal.draw(|f| {
    //     let size = f.size();
    //     let block = Block::default()
    //         .title("Block")
    //         .borders(Borders::ALL);
    //     f.render_widget(block, size);
    // });

    futures::future::join_all(handles).await;

    Ok(())
}
