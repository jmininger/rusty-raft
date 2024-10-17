use rand::Rng;
use tokio::{
    select,
    sync::{
        mpsc,
        oneshot,
    },
};
use tracing::warn;

#[allow(dead_code)]
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
