use tokio::sync::mpsc::Sender;
use tokio::time::{sleep, Duration};
use super::ShellEvent;

pub async fn watch(tx: Sender<ShellEvent>) {
    let mut count = 0u32;

    loop {
        count += 1;
        tx.send(ShellEvent::Dummy(count)).await.unwrap();
        sleep(Duration::from_secs(5)).await;
    }
}