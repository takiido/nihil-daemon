mod dbus;
mod providers;
mod system_features;
mod paths;

use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);

    let local = tokio::task::LocalSet::new();

    local.spawn_local(providers::dummy::watch(tx.clone()));
    local.spawn_local(providers::backlight::watch(tx.clone()));

    local.run_until(dbus::run(rx)).await;
}