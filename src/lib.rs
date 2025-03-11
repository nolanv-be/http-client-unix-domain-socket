//! # http-client-unix-domain-socket
//!
//! > A simple HTTP (json) client using UNIX domain socket in Rust
//!
//! This Rust crate provides a Unix socket HTTP client for asynchronous communication with local servers. It enables seamless interaction via Unix domain sockets using Tokio and Hyper, supporting raw HTTP requests and optional JSON serialization through feature flags. The client handles connection management, request building, response parsing, and error handling, including reconnection logic.
//!
//! ## Examples
//!
//! ### Simple GET request
//! ```rust
//! use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode};
//!
//! pub async fn get_hello_world() {
//!     let mut client = ClientUnix::try_new("/tmp/unix.socket")
//!         .await
//!         .expect("ClientUnix::try_new");
//!
//!     let (status_code, response) = client
//!         .send_request("/nolanv", Method::GET, &[], None)
//!         .await
//!         .expect("client.send_request");
//!
//!     assert_eq!(status_code, StatusCode::OK);
//!     assert_eq!(response, "Hello nolanv".as_bytes());
//! }
//! ```
//! ### Simple JSON POST request (feature=json)
//! ```rust
//! use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode};
//! #[cfg(feature = "json")]
//! use serde::{Deserialize, Serialize};
//! #[cfg(feature = "json")]
//! use serde_json::Value;
//!
//! #[cfg(feature = "json")]
//! #[derive(Serialize)]
//! struct NameJson {
//!     name: String,
//! }
//!
//! #[cfg(feature = "json")]
//! #[derive(Deserialize)]
//! struct HelloJson {
//!     hello: String,
//! }
//!
//! #[cfg(feature = "json")]
//! pub async fn get_hello_world() {
//!     let mut client = ClientUnix::try_new("/tmp/unix.socket")
//!         .await
//!         .expect("ClientUnix::try_new");
//!
//!     let (status_code, response) = client
//!         .send_request_json::<NameJson, HelloJson, Value>(
//!             "/nolanv",
//!             Method::POST,
//!             &[],
//!             Some(&NameJson {
//!                 name: "nolanv".into(),
//!             })
//!         )
//!         .await
//!         .expect("client.send_request_json");
//!
//!     assert_eq!(status_code, StatusCode::OK);
//!     assert_eq!(response.hello, "nolanv");
//! }
//! ```
//! ## Feature flags
//! - `json`: Add `send_request_json` which enable automatic parsing of request/response body with `serde_json` and add `Content-Type` header.

mod client;
mod error;
#[cfg(test)]
pub mod test_helpers;

pub use client::ClientUnix;
#[cfg(feature = "json")]
pub use error::ErrorAndResponseJson;
pub use error::{Error, ErrorAndResponse};
pub use hyper::Method;
pub use hyper::StatusCode;
