use bytes::Bytes;
use http::{HeaderMap, Method};
use reqwest::Client;
use rhedge::{HedgedClient, HedgedRequest};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    let hedged = HedgedClient::new(client, 2, Duration::from_millis(10), 0.95, 0.8);

    for _ in 0..10 {
        let req = HedgedRequest::new(
            Method::GET,
            "http://127.0.0.1:9999/hello".parse()?,
            HeaderMap::new(),
            Bytes::new(),
        );

        let resp = hedged.send(req).await?;
        println!("Status: {}", resp.status());
    }

    Ok(())
}
