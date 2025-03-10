//! # http-client-unix-domain-socket
//!
//! A simple HTTP (json) client using UNIX domain socket in Rust
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
//!
//! ### Unsuccessful response
//! ```rust
//! use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode, ErrorAndResponse};
//!
//! pub async fn get_path_not_found() {
//!     let mut client = ClientUnix::try_new("/tmp/unix.socket")
//!         .await
//!         .expect("ClientUnix::try_new");
//!
//!     let response_result = client
//!         .send_request("/nolanv", Method::GET, &[], None)
//!         .await;
//!
//!     assert!(matches!(
//!         response_result.err(),
//!         Some(ErrorAndResponse::ResponseUnsuccessful(status_code, _))
//!             if status_code == StatusCode::NOT_FOUND
//!     ));
//! }
//! ```
//!
//! ### Simple JSON GET request (feature=json)
//! ```rust
//! use http_client_unix_domain_socket::{JsonClientUnix, Method, StatusCode};
//! use serde::Deserialize;
//! use serde_json::Value;
//!
//! #[derive(Deserialize)]
//! struct HelloJson {
//!     hello: String,
//! }
//!
//! pub async fn get_hello_world() {
//!     let mut client = JsonClientUnix::try_new("/tmp/unix.socket")
//!         .await
//!         .expect("ClientUnix::try_new");
//!
//!     let (status_code, response) = client
//!         .send_request_json::<(), HelloJson, Value>(
//!             "/nolanv",
//!             Method::GET,
//!             &[],
//!             None
//!         )
//!         .await
//!         .expect("client.send_request_json");
//!
//!     assert_eq!(status_code, StatusCode::OK);
//!     assert_eq!(response.hello, "nolanv");
//! }
//! ```
//!
//! ### Simple JSON POST request (feature=json)
//! ```rust
//! use http_client_unix_domain_socket::{JsonClientUnix, Method, StatusCode};
//! use serde::{Deserialize, Serialize};
//! use serde_json::Value;
//!
//! #[derive(Serialize)]
//! struct NameJson {
//!     name: String,
//! }
//!
//! #[derive(Deserialize)]
//! struct HelloJson {
//!     hello: String,
//! }
//!
//! pub async fn get_hello_world() {
//!     let mut client = JsonClientUnix::try_new("/tmp/unix.socket")
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
//! - `json`: Add `JsonClientUnix` which enable automatic parsing of request/response body with `serde_json` and add `Content-type` header.

mod client;
#[cfg(feature = "json")]
mod client_json;
mod error;
#[cfg(test)]
pub mod test_helpers;

pub use client::ClientUnix;
#[cfg(feature = "json")]
pub use client_json::JsonClientUnix;
#[cfg(feature = "json")]
pub use error::ErrorAndResponseJson;
pub use error::{Error, ErrorAndResponse};
pub use hyper::Method;
pub use hyper::StatusCode;
