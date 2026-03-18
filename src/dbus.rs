use tokio::sync::mpsc::Receiver;
use zbus::{connection, interface, object_server::SignalEmitter};
use crate::providers::ShellEvent;

struct NihilShell;

#[interface(name = "org.nihil.Shell")]
impl NihilShell {
    #[zbus(signal)]
    async fn dummy_updated(ctx: &SignalEmitter<'_>, value: u32) -> zbus::Result<()>;
}

pub async fn run(mut rx: Receiver<ShellEvent>) {
    let conn = connection::Builder::session()
        .unwrap()
        .name("org.nihil.Shell")
        .unwrap()
        .serve_at("/org/nihil/Shell", NihilShell)
        .unwrap()
        .build()
        .await
        .unwrap();

    let iface = conn.object_server()
        .interface::<_, NihilShell>("/org/nihil/Shell")
        .await
        .unwrap();

    while let Some(event) = rx.recv().await {
        match event {
            ShellEvent::Dummy(n) => {
                println!("Emitting: {n}");
                NihilShell::dummy_updated(iface.signal_emitter(), n)
                    .await
                    .unwrap();
            }
        }
    }
}