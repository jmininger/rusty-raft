use color_eyre::Result;
use rand::Rng;
use tokio::{
    net::{
        unix::SocketAddr,
        TcpListener,
        TcpSocket,
        ToSocketAddrs,
    },
    select,
    sync::{
        mpsc,
        oneshot,
    },
};
use tracing::warn;

#[tokio::main]
async fn main() -> Result<()> {
    Ok(())
}

async fn start_network(addr: impl ToSocketAddrs) -> Result<()> {
    let listener = TcpListener::bind(addr).await?;

    while let Ok((sock, addr)) = listener.accept().await {
        todo!()
    }

    todo!()
}

async fn election_timeout(
    mut msg_alert: mpsc::Receiver<()>,
    election_trigger: oneshot::Sender<()>,
) {
    let mut rng = rand::thread_rng();
    loop {
        let rand_timeout = rng.gen_range(150..300);
        select! {
            _ = msg_alert.recv() => (),
            _ = tokio::time::sleep(std::time::Duration::from_millis(rand_timeout)) => break,
        }
    }
    if let Err(_) = election_trigger.send(()) {
        warn!("Election timeout triggered but failed to send warning via oneshot");
    }
}

/*
 * Who fires the first ever message?
 * - Broadcast (but each has a retry for failures)
 * Streams are only initiated on the first dial
 *
 */
