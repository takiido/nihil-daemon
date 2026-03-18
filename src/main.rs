mod dbus;
mod providers;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    tokio::spawn(providers::dummy::watch(tx.clone()));

    dbus::run(rx).await;
}