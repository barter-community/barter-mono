use std::time::Duration;

use tokio::{sync::mpsc, time::sleep};

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::unbounded_channel::<String>();

    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(1)).await;
            tx.send("hello".to_string()).unwrap();
        }
    });

    // listen(rx).await.unwrap();
    listen2(rx);
    // .join().unwrap();
}

fn listen(mut rx: mpsc::UnboundedReceiver<String>) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            println!("Received: {}", msg);
        }
    })
}

fn listen2(mut rx: mpsc::UnboundedReceiver<String>) -> std::thread::JoinHandle<()> {
    std::thread::spawn(move || loop {
        match rx.try_recv() {
            Ok(msg) => println!("Received: {}", msg),
            Err(err) => {
                // println!("Error: {:?}", err);
                continue;
            }
        }
    })
}
