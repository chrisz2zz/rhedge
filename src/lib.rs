//! # rhedge
//!
//! A hedged request library that sends redundant HTTP requests to reduce tail latency.
//!
//! Inspired by [Hedged Requests](https://research.google/pubs/pub40801/) from Google's
//! "The Tail at Scale" paper, this library monitors historical response latencies and
//! automatically dispatches hedge requests when the primary request is slower than expected.

pub mod client;
pub mod digest;
pub mod error;
pub mod request;

pub use client::HedgedClient;
pub use error::HedgedError;
pub use request::HedgedRequest;
