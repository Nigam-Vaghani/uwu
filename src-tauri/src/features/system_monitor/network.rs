use std::time::Instant;
use sysinfo::Networks;

#[derive(Debug, Clone, Copy, Default)]
pub struct NetworkRates {
    pub send_kbps: f64,
    pub recv_kbps: f64,
}

pub struct NetworkTracker {
    networks: Networks,
    last_total_sent: u64,
    last_total_recv: u64,
    last_sample: Option<Instant>,
}

impl NetworkTracker {
    pub fn new() -> Self {
        let mut networks = Networks::new_with_refreshed_list();
        networks.refresh(true);

        let (sent, recv) = total_bytes(&networks);
        Self {
            networks,
            last_total_sent: sent,
            last_total_recv: recv,
            last_sample: Some(Instant::now()),
        }
    }

    pub fn poll(&mut self) -> NetworkRates {
        self.networks.refresh(true);
        let (sent, recv) = total_bytes(&self.networks);
        let now = Instant::now();

        let rates = match self.last_sample {
            Some(previous) => {
                let elapsed = now.duration_since(previous).as_secs_f64().max(0.001);
                let sent_delta = sent.saturating_sub(self.last_total_sent) as f64;
                let recv_delta = recv.saturating_sub(self.last_total_recv) as f64;
                NetworkRates {
                    send_kbps: (sent_delta / 1024.0) / elapsed,
                    recv_kbps: (recv_delta / 1024.0) / elapsed,
                }
            }
            None => NetworkRates::default(),
        };

        self.last_total_sent = sent;
        self.last_total_recv = recv;
        self.last_sample = Some(now);
        rates
    }
}

fn total_bytes(networks: &Networks) -> (u64, u64) {
    networks
        .iter()
        .fold((0u64, 0u64), |(sent, recv), (_, data)| {
            (
                sent.saturating_add(data.total_transmitted()),
                recv.saturating_add(data.total_received()),
            )
        })
}
