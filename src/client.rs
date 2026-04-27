use futures::stream::{FuturesUnordered, StreamExt};
use reqwest::Client;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

use crate::digest::LatencyDigest;
use crate::error::HedgedError;
use crate::request::HedgedRequest;

#[derive(Clone)]
pub struct HedgedClient {
    client: Client,
    digest: Arc<LatencyDigest>,

    max_hedges: usize,
    min_delay: Duration,
    quantile: f64,
    multiplier: f64,
}

impl HedgedClient {
    pub fn new(
        client: Client,
        max_hedges: usize,
        min_delay: Duration,
        quantile: f64,
        multiplier: f64,
    ) -> Self {
        Self {
            client,
            digest: Arc::new(LatencyDigest::new(100)),
            max_hedges,
            min_delay,
            quantile,
            multiplier,
        }
    }

    pub async fn send(&self, req_tmpl: HedgedRequest) -> Result<reqwest::Response, HedgedError> {
        let start = std::time::Instant::now();

        let delay = self.current_delay();

        let mut pending: FuturesUnordered<
            tokio::task::JoinHandle<Result<reqwest::Response, HedgedError>>,
        > = FuturesUnordered::new();

        // 发送初始请求
        let initial_req = match req_tmpl.to_reqwest(&self.client) {
            Ok(req) => req,
            Err(e) => return Err(HedgedError::Build(e)),
        };
        pending.push(tokio::spawn({
            let client = self.client.clone();
            async move {
                client
                    .execute(initial_req)
                    .await
                    .map_err(HedgedError::Request)
            }
        }));

        let mut hedges_sent = 0;

        loop {
            tokio::select! {
                Some(res) = pending.next() => {
                    let elapsed = start.elapsed();
                    self.record_latency(elapsed);
                    match res {
                        Ok(Ok(response)) => return Ok(response),
                        Ok(Err(e)) => return Err(e),
                        Err(_) => {
                            if hedges_sent == 0 && pending.is_empty() {
                                return Err(HedgedError::AllTasksPanicked);
                            }
                            continue;
                        }
                    }
                }
                _ = sleep(delay), if hedges_sent < self.max_hedges => {
                    hedges_sent += 1;
                    let tmpl = req_tmpl.clone();
                    let client = self.client.clone();
                    pending.push(tokio::spawn(async move {
                        match tmpl.to_reqwest(&client) {
                            Ok(req) => client.execute(req).await.map_err(HedgedError::Request),
                            Err(e) => Err(HedgedError::Build(e)),
                        }
                    }));
                }
            }
        }
    }

    fn current_delay(&self) -> Duration {
        let p = self.digest.percentile(self.quantile);
        if p <= 0.0 {
            self.min_delay
        } else {
            let raw = Duration::from_micros(p as u64);
            let scaled = raw.mul_f64(self.multiplier);
            scaled.max(self.min_delay)
        }
    }

    fn record_latency(&self, d: Duration) {
        self.digest.record(d.as_micros() as f64);
    }
}
