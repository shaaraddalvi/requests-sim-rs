use circular_queue::CircularQueue;
use rand::{distributions::Uniform, thread_rng, Rng};
use std::{
    error::Error,
    io,
    sync::{Arc, Mutex},
    vec::Vec,
};
use stopwatch::Stopwatch;
use termion::raw::IntoRawMode;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
    stream::StreamExt,
    sync::mpsc,
    time::{sleep, Duration},
};
use tui::{
    backend::TermionBackend,
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{Axis, Block, Borders, Chart, Dataset},
    Terminal,
};

/// Child task that takes a connection, simulates artificial delay and sends the repsponse
async fn process_stream(
    mut stream: tokio::net::TcpStream,
    tx: mpsc::Sender<isize>,
) -> Result<(), Box<dyn Error>> {
    let wait_duration = thread_rng().sample(Uniform::new(1000u64, 20000));

    tx.send(1).await?;
    let buf = stream.read_u8().await;

    match buf {
        Ok(_) => {
            sleep(Duration::from_millis(wait_duration)).await;
            match stream.write_all("Got".as_bytes()).await {
                Ok(_) => {
                    tx.send(-1).await?;
                }
                Err(_) => {}
            }
        }
        Err(_) => {}
    }

    Ok(())
}

/// Main task that binds to port and gives out incoming connections to children tasks
async fn handle_tcp(tx: mpsc::Sender<isize>) -> Result<(), Box<dyn Error>> {
    let mut listener = TcpListener::bind("127.0.0.1:8900").await.unwrap();
    while let Some(stream) = listener.next().await {
        match stream {
            Ok(stream) => {
                let tx2 = tx.clone();
                tokio::spawn(async move {
                    match process_stream(stream, tx2).await {
                        Ok(_) => {}
                        Err(_) => {}
                    }
                });
            }
            Err(_) => { /* connection failed */ }
        }
    }

    Ok(())
}

/// Counter - Receives increments/decrements from channel and updates the value into mutex
async fn counter(
    mut rx: mpsc::Receiver<isize>,
    counter_handle: Arc<Mutex<isize>>,
) -> Result<(), Box<dyn Error>> {
    while let Some(message) = rx.recv().await {
        *counter_handle.lock().unwrap() += message;
    }

    Ok(())
}

/// Displayer - Every second gets the current in-flight request value from mutex
/// and updates the UI
async fn displayer(counter_handle: Arc<Mutex<isize>>) -> Result<(), Box<dyn Error>> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let sw = Stopwatch::start_new();
    let mut queue = CircularQueue::<(f64, f64)>::with_capacity(250);

    loop {
        sleep(Duration::from_millis(1000)).await;

        let current_count = counter_handle.lock().unwrap().clone();
        queue.push(((sw.elapsed_ms() / 1000) as f64, current_count as f64));
        let mut data_vec = Vec::<(f64, f64)>::new();
        let mut min_x = f64::MAX;
        let mut max_x = 0.0;
        let mut max_y = 0.0;
        let mut x_labels = Vec::<Span>::new();
        let mut y_labels = Vec::<Span>::new();
        for (i, data) in queue.iter().enumerate() {
            if data.0 < min_x {
                min_x = data.0
            }
            if data.0 > max_x {
                max_x = data.0
            }
            if data.1 > max_y {
                max_y = data.1
            }
            if i % 10 == 0 {
                x_labels.push(Span::raw(data.0.to_string()));
            }
            data_vec.push(data.clone());
        }
        x_labels.reverse();
        for mul in [0.0, 0.25, 0.5, 0.75, 1.0, 1.5].iter() {
            y_labels.push(Span::raw((max_y * mul).to_string()));
        }

        let datasets = vec![Dataset::default()
            .name("Requests")
            .marker(symbols::Marker::Dot)
            .style(Style::default().fg(Color::Cyan))
            .data(&data_vec)];

        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().title("Monitor").borders(Borders::ALL);
            let chart = Chart::new(datasets)
                .block(block)
                .x_axis(
                    Axis::default()
                        .title("Time")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([min_x, max_x])
                        .labels(x_labels),
                )
                .y_axis(
                    Axis::default()
                        .title("In-flight requests")
                        .style(Style::default().fg(Color::Gray))
                        .bounds([0.0, max_y * 1.5])
                        .labels(y_labels),
                );
            f.render_widget(chart, size);
        })?;
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<isize>(32);

    let mut handles = Vec::new();

    handles.push(tokio::spawn(async {
        match handle_tcp(tx).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }));

    let counter_arc = Arc::new(Mutex::<isize>::new(0));

    let counter_clone = counter_arc.clone();
    handles.push(tokio::spawn(async {
        match counter(rx, counter_clone).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }));

    let displayer_clone = counter_arc.clone();
    handles.push(tokio::spawn(async {
        match displayer(displayer_clone).await {
            Ok(_) => {}
            Err(_) => {}
        }
    }));

    futures::future::join_all(handles).await;

    Ok(())
}
