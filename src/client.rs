#[cfg(feature = "json")]
use crate::error::ErrorAndResponseJson;
use crate::{Error, error::ErrorAndResponse};
use axum_core::body::Body;
use http_body_util::BodyExt;
use hyper::{
    Method, Request, StatusCode,
    client::conn::http1::{self, SendRequest},
};
use hyper_util::rt::TokioIo;
#[cfg(feature = "json")]
use serde::{Serialize, de::DeserializeOwned};
use std::path::PathBuf;
use tokio::{net::UnixStream, task::JoinHandle};

/// A simple HTTP (json) client using UNIX domain socket in Rust
#[derive(Debug)]
pub struct ClientUnix {
    socket_path: PathBuf,
    sender: SendRequest<Body>,
    join_handle: JoinHandle<Error>,
}

impl ClientUnix {
    /// Create a new HTTP client and try to connect to it.
    ///
    /// # Example
    /// ```rust
    /// use http_client_unix_domain_socket::ClientUnix;
    ///
    /// pub async fn new_client() {
    ///     ClientUnix::try_new("/tmp/unix.socket").await.expect("ClientUnix::try_new");
    /// }
    /// ```
    pub async fn try_new(socket_path: &str) -> Result<Self, Error> {
        let socket_path = PathBuf::from(socket_path);
        ClientUnix::try_connect(socket_path).await
    }

    /// Reconnect to an existing [ClientUnix].
    ///
    /// Sometimes the server to which the client is connected may reboot, causing the client to disconnect. For simplicity, no automatic reconnection is implemented - it must be manually performed by calling this function.
    /// The error will be probably trigger during the [ClientUnix::send_request](or [ClientUnix::send_request_json]) with this error [Error::RequestSend].
    /// # Example
    /// ```rust
    /// use http_client_unix_domain_socket::{ClientUnix, Method, Error, ErrorAndResponse};
    ///
    /// pub async fn reconnect_after_failure() {
    ///     let mut client = ClientUnix::try_new("/tmp/unix.socket").await.expect("ClientUnix::try_new");
    ///     let response_result = client.send_request("/nolanv", Method::GET, &[], None).await;
    ///
    ///     if(matches!(
    ///         response_result.err(),
    ///         Some(ErrorAndResponse::InternalError(Error::RequestSend(e)))
    ///             if e.is_canceled()
    ///     )){
    ///         client = client.try_reconnect().await.expect("client.try_reconnect");
    ///     }
    /// }
    /// ```
    pub async fn try_reconnect(self) -> Result<Self, Error> {
        let socket_path = self.socket_path.clone();
        self.abort().await;
        ClientUnix::try_connect(socket_path).await
    }

    /// Abort the [ClientUnix] connection [JoinHandle].
    ///
    /// Used for stopping the connection [JoinHandle]([tokio::task]), it's also used for [ClientUnix::try_reconnect]. The returned [Error] can be used to know if it was stopped without any error.
    pub async fn abort(self) -> Option<Error> {
        self.join_handle.abort();
        self.join_handle.await.ok()
    }

    async fn try_connect(socket_path: PathBuf) -> Result<Self, Error> {
        let stream = TokioIo::new(
            UnixStream::connect(socket_path.clone())
                .await
                .map_err(Error::SocketConnectionInitiation)?,
        );

        let (sender, connection) = http1::handshake(stream).await.map_err(Error::Handhsake)?;

        let join_handle =
            tokio::task::spawn(
                async move { Error::SocketConnectionClosed(connection.await.err()) },
            );

        Ok(ClientUnix {
            socket_path,
            sender,
            join_handle,
        })
    }

    /// Send a raw HTTP request.
    ///
    /// The [ClientUnix::send_request] method allows sending an HTTP request without serializing it. This method can be useful when communicating using a format other than JSON, or for endpoints that don’t return responses adhering to the JSON format. [Error] are wrapped in an Enum [ErrorAndResponse] that includes both [ErrorAndResponse::InternalError] and HTTP response [ErrorAndResponse::ResponseUnsuccessful].
    /// # Examples
    /// ## HTTP GET
    /// ```rust
    /// use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode, ErrorAndResponse};
    ///
    /// pub async fn get_hello_world() {
    ///     let mut client = ClientUnix::try_new("/tmp/unix.socket")
    ///         .await
    ///         .expect("ClientUnix::try_new");
    ///
    ///     match client
    ///         .send_request("/nolanv", Method::GET, &vec![("Host", "localhost")], None)
    ///         .await
    ///     {
    ///         Err(ErrorAndResponse::ResponseUnsuccessful(status_code, response)) => {
    ///             assert!(status_code == StatusCode::NOT_FOUND);
    ///             assert!(response == "not found".as_bytes());
    ///         }
    ///
    ///         Ok((status_code, response)) => {
    ///             assert_eq!(status_code, StatusCode::OK);
    ///             assert_eq!(response, "Hello nolanv".as_bytes());
    ///         }
    ///
    ///         Err(_) => panic!("Something went wrong")
    ///     }
    /// }
    /// ```
    /// ## HTTP POST
    /// ```rust
    /// use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode, Body};
    ///
    /// pub async fn post_hello_world() {
    ///     let mut client = ClientUnix::try_new("/tmp/unix.socket")
    ///         .await
    ///         .expect("ClientUnix::try_new");
    ///
    ///     let (status_code, response) = client
    ///         .send_request("/", Method::POST, &[], Some(Body::from("nolanv")))
    ///         .await
    ///         .expect("client.send_request");
    ///
    ///     assert_eq!(status_code, StatusCode::OK);
    ///     assert_eq!(response, "Hello nolanv".as_bytes());
    /// }
    /// ```
    pub async fn send_request(
        &mut self,
        endpoint: &str,
        method: Method,
        headers: &[(&str, &str)],
        body_request: Option<Body>,
    ) -> Result<(StatusCode, Vec<u8>), ErrorAndResponse> {
        let mut request_builder = Request::builder();
        for header in headers {
            request_builder = request_builder.header(header.0, header.1);
        }
        let request = request_builder
            .method(method)
            .uri(format!("http://unix.socket{}", endpoint))
            .body(body_request.unwrap_or(Body::empty()))
            .map_err(|e| ErrorAndResponse::InternalError(Error::RequestBuild(e)))?;

        let response = self
            .sender
            .send_request(request)
            .await
            .map_err(|e| ErrorAndResponse::InternalError(Error::RequestSend(e)))?;

        let status_code = response.status();
        let body_response = response
            .collect()
            .await
            .map_err(|e| ErrorAndResponse::InternalError(Error::ResponseCollect(e)))?
            .to_bytes();

        if !status_code.is_success() {
            return Err(ErrorAndResponse::ResponseUnsuccessful(
                status_code,
                body_response.to_vec(),
            ));
        }
        Ok((status_code, body_response.to_vec()))
    }

    /// Send JSON HTTP request **(feature = json)**
    ///
    /// Use [ClientUnix::send_request], adding automatically the "Content-Type" header and handling JSON (de)serialization for both the request body and response. This method does not use the same [Error] Enum, enabling typed error responses instead via [ErrorAndResponseJson].
    /// # Examples
    /// ## HTTP POST JSON **(feature = json)**
    /// ```rust
    /// use http_client_unix_domain_socket::{ClientUnix, Method, StatusCode, ErrorAndResponseJson};
    /// use serde::{Deserialize, Serialize};
    ///
    /// #[derive(Serialize)]
    /// struct NameJson {
    ///     name: String,
    /// }
    ///
    /// #[derive(Deserialize)]
    /// struct HelloJson {
    ///     hello: String,
    /// }
    ///
    /// #[derive(Deserialize, Debug)]
    /// struct ErrorJson {
    ///     msg: String,
    /// }
    ///
    /// pub async fn post_hello_world() {
    ///     let mut client = ClientUnix::try_new("/tmp/unix.socket")
    ///         .await
    ///         .expect("ClientUnix::try_new");
    ///
    ///     match client
    ///         .send_request_json::<NameJson, HelloJson, ErrorJson>(
    ///             "/nolanv",
    ///             Method::POST,
    ///             &vec![("Host", "localhost")],
    ///             Some(&NameJson { name: "nolanv".into() }))
    ///         .await
    ///     {
    ///         Err(ErrorAndResponseJson::ResponseUnsuccessful(status_code, response)) => {
    ///             assert!(status_code == StatusCode::BAD_REQUEST);
    ///             assert!(response.msg == "bad request");
    ///         }
    ///
    ///         Ok((status_code, response)) => {
    ///             assert_eq!(status_code, StatusCode::OK);
    ///             assert_eq!(response.hello, "nolanv");
    ///         }
    ///
    ///         Err(_) => panic!("Something went wrong")
    ///     }
    /// }
    /// ```
    #[cfg(feature = "json")]
    pub async fn send_request_json<IN: Serialize, OUT: DeserializeOwned, ERR: DeserializeOwned>(
        &mut self,
        endpoint: &str,
        method: Method,
        headers: &[(&str, &str)],
        body_request: Option<&IN>,
    ) -> Result<(StatusCode, OUT), ErrorAndResponseJson<ERR>> {
        let mut headers = headers.to_vec();
        headers.push(("Content-Type", "application/json"));

        let body_request = match body_request {
            Some(body_request) => Body::from(
                serde_json::to_vec(body_request)
                    .map_err(|e| ErrorAndResponseJson::InternalError(Error::RequestParsing(e)))?,
            ),
            None => Body::empty(),
        };

        match self
            .send_request(endpoint, method, &headers, Some(body_request))
            .await
        {
            Ok((status_code, response)) => Ok((
                status_code,
                serde_json::from_slice(&response)
                    .map_err(|e| ErrorAndResponseJson::InternalError(Error::ResponseParsing(e)))?,
            )),
            Err(ErrorAndResponse::InternalError(e)) => Err(ErrorAndResponseJson::InternalError(e)),
            Err(ErrorAndResponse::ResponseUnsuccessful(status_code, response)) => {
                Err(ErrorAndResponseJson::ResponseUnsuccessful(
                    status_code,
                    serde_json::from_slice(&response).map_err(|e| {
                        ErrorAndResponseJson::InternalError(Error::ResponseParsing(e))
                    })?,
                ))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{server::Server, util::*};
    use hyper::Method;

    #[tokio::test]
    async fn simple_request() {
        let (_, mut client) = make_client_server("simple_request").await;

        let (status_code, response) = client
            .send_request("/nolanv", Method::GET, &[], None)
            .await
            .expect("client.send_request");

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response, "Hello nolanv".as_bytes())
    }

    #[tokio::test]
    async fn simple_404_request() {
        let (_, mut client) = make_client_server("simple_404_request").await;

        let result = client
            .send_request("/nolanv/nope", Method::GET, &[], None)
            .await;

        assert!(matches!(
            result.err(),
            Some(ErrorAndResponse::ResponseUnsuccessful(status_code, _))
                if status_code == StatusCode::NOT_FOUND
        ));
    }

    #[tokio::test]
    async fn multiple_request() {
        let (_, mut client) = make_client_server("multiple_request").await;

        for i in 0..20 {
            let (status_code, response) = client
                .send_request(&format!("/nolanv{}", i), Method::GET, &[], None)
                .await
                .expect("client.send_request");

            assert_eq!(status_code, StatusCode::OK);

            assert_eq!(response, format!("Hello nolanv{}", i).as_bytes())
        }
    }

    #[tokio::test]
    async fn server_not_started() {
        let socket_path = make_socket_path_test("client", "server_not_started");

        let client = ClientUnix::try_new(&socket_path).await;
        assert!(matches!(
            client.err(),
            Some(Error::SocketConnectionInitiation(_))
        ));
    }

    #[tokio::test]
    async fn server_stopped() {
        let (server, mut client) = make_client_server("server_stopped").await;
        server.abort().await;

        let response_result = client.send_request("/nolanv", Method::GET, &[], None).await;
        assert!(matches!(
            response_result.err(),
                         Some(ErrorAndResponse::InternalError(Error::RequestSend(e)))
                         if e.is_canceled()
        ));

        let _ = Server::try_new(&make_socket_path_test("client", "server_stopped"))
            .await
            .expect("Server::try_new");
        let mut http_client = client.try_reconnect().await.expect("client.try_reconnect");

        let (status_code, response) = http_client
            .send_request("/nolanv", Method::GET, &[], None)
            .await
            .expect("client.send_request");

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response, "Hello nolanv".as_bytes())
    }

    #[tokio::test]
    async fn server_rebooted() {
        let (server, mut client) = make_client_server("server_rebooted").await;
        server.abort().await;

        let _ = Server::try_new(&make_socket_path_test("client", "server_rebooted"))
            .await
            .expect("Server::try_new");

        let response_result = client.send_request("/nolanv", Method::GET, &[], None).await;
        assert!(matches!(
            response_result.err(),
                         Some(ErrorAndResponse::InternalError(Error::RequestSend(e)))
                         if e.is_canceled()
        ));
        let mut http_client = client.try_reconnect().await.expect("client.try_reconnect");

        let (status_code, response) = http_client
            .send_request("/nolanv", Method::GET, &[], None)
            .await
            .expect("client.send_request");

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response, "Hello nolanv".as_bytes())
    }
}

#[cfg(feature = "json")]
#[cfg(test)]
mod json_tests {
    use hyper::{Method, StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    use crate::{error::ErrorAndResponseJson, test_helpers::util::make_client_server};

    #[derive(Deserialize, Debug)]
    struct ErrorJson {
        msg: String,
    }

    #[tokio::test]
    async fn simple_get_request() {
        let (_, mut client) = make_client_server("simple_get_request").await;

        let (status_code, response) = client
            .send_request_json::<(), Value, Value>("/json/nolanv", Method::GET, &[], None)
            .await
            .expect("client.send_request_json");

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response.get("hello"), Some(&json!("nolanv")))
    }

    #[tokio::test]
    async fn simple_get_404_request() {
        let (_, mut client) = make_client_server("simple_get_404_request").await;

        let result = client
            .send_request_json::<(), Value, ErrorJson>("/json/nolanv/nop", Method::GET, &[], None)
            .await;

        dbg!(&result);
        assert!(matches!(
            result.err(),
                         Some(ErrorAndResponseJson::ResponseUnsuccessful(status_code, body))
                         if status_code == StatusCode::NOT_FOUND && body.msg == "not found"
        ));
    }

    #[tokio::test]
    async fn simple_post_request() {
        let (_, mut client) = make_client_server("simple_post_request").await;

        #[derive(Serialize)]
        struct NameJson {
            name: String,
        }

        #[derive(Deserialize)]
        struct HelloJson {
            hello: String,
        }

        let (status_code, response) = client
            .send_request_json::<NameJson, HelloJson, Value>(
                "/json",
                Method::POST,
                &[],
                Some(&NameJson {
                    name: "nolanv".into(),
                }),
            )
            .await
            .expect("client.send_request_json");

        assert_eq!(status_code, StatusCode::OK);
        assert_eq!(response.hello, "nolanv")
    }

    #[tokio::test]
    async fn simple_post_bad_request() {
        let (_, mut client) = make_client_server("simple_post_bad_request").await;

        #[derive(Serialize)]
        struct NameBadJson {
            nom: String,
        }

        let result = client
            .send_request_json::<NameBadJson, Value, ErrorJson>(
                "/json",
                Method::POST,
                &[],
                Some(&NameBadJson {
                    nom: "nolanv".into(),
                }),
            )
            .await;

        assert!(matches!(
            result.err(),
                         Some(ErrorAndResponseJson::ResponseUnsuccessful(status_code, body))
                         if status_code == StatusCode::BAD_REQUEST && body.msg == "bad request"
        ));
    }
}
