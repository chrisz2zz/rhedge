//! Request template for hedged HTTP requests.

use bytes::Bytes;
use http::{HeaderMap, Method, Uri};
use reqwest::Client;
use std::sync::Arc;

/// A reusable HTTP request template that can be converted into a [`reqwest::Request`].
///
/// `HedgedRequest` stores the request components (method, URL, headers, body) so that
/// multiple identical requests can be dispatched without rebuilding the template each time.
///
/// Internally, expensive-to-clone fields (`method`, `url`, `headers`) are wrapped in
/// [`Arc`], making [`Clone`] an O(1) reference-count increment. The `body` field uses
/// [`Bytes`], which is also O(1) to clone. The URL string representation is cached to
/// avoid repeated [`Uri::to_string`] allocations when building requests.
pub struct HedgedRequest {
    method: Arc<Method>,
    url: Arc<Uri>,
    /// Cached string representation of `url` to avoid repeated allocations.
    url_str: Arc<String>,
    headers: Arc<HeaderMap>,
    body: Bytes,
}

impl Clone for HedgedRequest {
    fn clone(&self) -> Self {
        Self {
            method: Arc::clone(&self.method),
            url: Arc::clone(&self.url),
            url_str: Arc::clone(&self.url_str),
            headers: Arc::clone(&self.headers),
            body: self.body.clone(),
        }
    }
}

impl HedgedRequest {
    /// Creates a new `HedgedRequest` from the given components.
    pub fn new(method: Method, url: Uri, headers: HeaderMap, body: Bytes) -> Self {
        let url_str = url.to_string();
        Self {
            method: Arc::new(method),
            url: Arc::new(url),
            url_str: Arc::new(url_str),
            headers: Arc::new(headers),
            body,
        }
    }

    /// Returns a reference to the HTTP method.
    pub fn method(&self) -> &Method {
        &self.method
    }

    /// Returns a reference to the request URI.
    pub fn url(&self) -> &Uri {
        &self.url
    }

    /// Returns a reference to the request headers.
    pub fn headers(&self) -> &HeaderMap {
        &self.headers
    }

    /// Returns a reference to the request body.
    pub fn body(&self) -> &Bytes {
        &self.body
    }

    /// Converts this template into a [`reqwest::Request`] using the given client.
    ///
    /// Returns an error if the request cannot be built (e.g. invalid URL).
    pub fn to_reqwest(&self, client: &Client) -> Result<reqwest::Request, reqwest::Error> {
        client
            .request(self.method.as_ref().clone(), self.url_str.as_str())
            .headers(self.headers.as_ref().clone())
            .body(self.body.clone())
            .build()
    }
}
