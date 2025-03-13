# http-client-unix-domain-socket
[![Crates.io](https://img.shields.io/crates/v/http-client-unix-domain-socket)](https://crates.io/crates/http-client-unix-domain-socket)
[![Documentation](https://docs.rs/http-client-unix-domain-socket/badge.svg)][docs]

> A simple HTTP (json) client using UNIX domain socket in Rust

This Rust crate provides a Unix socket HTTP client for asynchronous communication with local servers. It enables seamless interaction via Unix domain sockets using Tokio and Hyper, supporting raw HTTP requests and optional JSON serialization through feature flags. The client handles connection management, request building, response parsing, and error handling, including reconnection logic.

More information about this crate can be found in the [crate documentation][docs].

## Examples

### HTTP GET
```rust
use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode, ErrorAndResponse};

pub async fn get_hello_world() {
    let mut client = ClientUnix::try_new("/tmp/unix.socket")
        .await
        .expect("ClientUnix::try_new");

    match client
        .send_request("/nolanv", Method::GET, &vec![("Host", "localhost")], None)
        .await
    {
        Err(ErrorAndResponse::ResponseUnsuccessful(status_code, response)) => {
            assert!(status_code == StatusCode::NOT_FOUND);
            assert!(response == "not found".as_bytes());
        }

        Ok((status_code, response)) => {
            assert_eq!(status_code, StatusCode::OK);
            assert_eq!(response, "Hello nolanv".as_bytes());
        }

        Err(_) => panic!("Something went wrong")
    }
}
```
### HTTP POST JSON **(feature = json)**
```rust
use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode, ErrorAndResponseJson};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
struct NameJson {
    name: String,
}

#[derive(Deserialize)]
struct HelloJson {
    hello: String,
}

#[derive(Deserialize, Debug)]
struct ErrorJson {
    msg: String,
}

pub async fn post_hello_world() {
    let mut client = ClientUnix::try_new("/tmp/unix.socket")
        .await
        .expect("ClientUnix::try_new");

    match client
        .send_request_json::<NameJson, HelloJson, ErrorJson>(
            "/nolanv",
            Method::POST,
            &vec![("Host", "localhost")],
            Some(&NameJson { name: "nolanv".into() }))
        .await
    {
        Err(ErrorAndResponseJson::ResponseUnsuccessful(status_code, response)) => {
            assert!(status_code == StatusCode::BAD_REQUEST);
            assert!(response.msg == "bad request");
        }

        Ok((status_code, response)) => {
            assert_eq!(status_code, StatusCode::OK);
            assert_eq!(response.hello, "nolanv");
        }

        Err(_) => panic!("Something went wrong")
    }
}
 ```
## Feature flags
- `json`(default): Add `send_request_json` which enable automatic parsing of request/response body with `serde_json` and add `Content-Type` header.

[docs]: https://docs.rs/http-client-unix-domain-socket
