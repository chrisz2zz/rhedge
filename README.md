# rhedge

A hedged request library for Rust that sends redundant HTTP requests to reduce tail latency.

Inspired by [Hedged Requests](https://research.google/pubs/pub40801/) from Google's *"The Tail at Scale"* paper, `rhedge` monitors historical response latencies and automatically dispatches hedge requests when the primary request is slower than expected.

## How It Works

1. Send the primary HTTP request.
2. Wait for an **adaptive delay** — computed from the historical latency percentile multiplied by a configurable factor.
3. If the primary request hasn't returned before the delay expires, send a **hedge request** (up to `max_hedges` times).
4. Return the **first successful response**; cancel the remaining in-flight requests.

The adaptive delay shrinks when the service is fast and grows when it's slow, keeping hedge overhead minimal under normal conditions while still protecting against tail latency spikes.

## Quick Start

Add `rhedge` to your `Cargo.toml`:

```toml
[dependencies]
rhedge = "0.1"
```

Then use it in your code:

```rust
use bytes::Bytes;
use http::{HeaderMap, Method};
use reqwest::Client;
use rhedge::{HedgedClient, HedgedRequest};
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // max_hedges = 2, min_delay = 10ms, quantile = 95th, multiplier = 0.8
    let hedged = HedgedClient::new(client, 2, Duration::from_millis(10), 0.95, 0.8);

    let req = HedgedRequest::new(
        Method::GET,
        "http://example.com/api".parse()?,
        HeaderMap::new(),
        Bytes::new(),
    );

    let resp = hedged.send(req).await?;
    println!("Status: {}", resp.status());

    Ok(())
}
```

## Configuration

`HedgedClient::new` accepts five parameters:

| Parameter | Type | Description |
|---|---|---|
| `client` | `reqwest::Client` | The underlying HTTP client used to execute requests. |
| `max_hedges` | `usize` | Maximum number of hedge requests to send per call. |
| `min_delay` | `Duration` | Minimum delay before sending the first hedge request. Acts as a floor for the adaptive delay. |
| `quantile` | `f64` | Percentile (0–1) of historical latency used to compute the hedge delay. E.g. `0.95` uses the 95th percentile. |
| `multiplier` | `f64` | Factor applied to the percentile latency. E.g. `0.8` means "send a hedge if the primary hasn't responded within 80% of the historical p95." |

### Choosing Parameters

- **`max_hedges`**: Start with 1–2. Higher values reduce tail latency further but increase server load.
- **`min_delay`**: Set to a value slightly below your typical fast response time (e.g. 5–20 ms). This prevents premature hedging when the digest has no data yet.
- **`quantile`**: `0.95` is a good default. Higher percentiles make hedging more aggressive.
- **`multiplier`**: Values below 1.0 cause hedging to trigger *before* the percentile threshold, providing extra safety margin. Values above 1.0 make hedging more conservative.

## Architecture

- **`HedgedClient`** — The main entry point. Wraps a `reqwest::Client` and manages the hedge lifecycle.
- **`HedgedRequest`** — A reusable request template with `Arc`-wrapped fields for cheap cloning. Supports conversion to `reqwest::Request` via `to_reqwest()`.
- **`LatencyDigest`** — A thread-safe T-Digest that records observed latencies and provides percentile estimates for adaptive delay calculation.
- **`HedgedError`** — Error type covering build failures, request failures, and task panics.

## License

This project is licensed under the MIT License.
