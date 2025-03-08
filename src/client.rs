use crate::Error;
use hyper::{
    Response,
    body::Incoming,
    client::conn::http1::{self, SendRequest},
};
use hyper_util::rt::TokioIo;
use std::path::PathBuf;
use tokio::{net::UnixStream, task::JoinHandle};

pub struct HttpClientUnixDomainSocket<B>
where
    B: hyper::body::Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    socket_path: PathBuf,
    sender: SendRequest<B>,
    join_handle: JoinHandle<Error>,
}

impl<B> HttpClientUnixDomainSocket<B>
where
    B: hyper::body::Body + 'static + Send,
    B::Data: Send,
    B::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    pub async fn try_new(socket_path: &str) -> Result<Self, Error> {
        let socket_path = PathBuf::from(socket_path);
        HttpClientUnixDomainSocket::try_connect(socket_path).await
    }

    pub async fn try_reconnect(self) -> Result<Self, Error> {
        let socket_path = self.socket_path.clone();
        self.abort().await;
        HttpClientUnixDomainSocket::try_connect(socket_path).await
    }

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

        Ok(HttpClientUnixDomainSocket {
            socket_path,
            sender,
            join_handle,
        })
    }

    pub async fn send_request(
        &mut self,
        request: hyper::Request<B>,
    ) -> Result<Response<Incoming>, Error> {
        self.sender
            .send_request(request)
            .await
            .map_err(Error::SendRequest)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{server::Server, util::*};
    use axum_core::body::Body;
    use hyper::{Method, Request};

    #[tokio::test]
    async fn simple_request() {
        let (_, mut client) = make_client_server("simple_request").await;

        let response = client
            .send_request(
                Request::builder()
                    .method(Method::GET)
                    .uri("http://unix.socket/nolanv")
                    .body(Body::empty())
                    .expect("Request builder"),
            )
            .await
            .expect("Response");

        assert_eq!(response.status().as_str(), "200");
        assert_eq!(
            response_to_utf8(response).await,
            Some("Hello nolanv".into())
        )
    }

    #[tokio::test]
    async fn multiple_request() {
        let (_, mut client) = make_client_server("multiple_request").await;

        for i in 0..20 {
            let response = client
                .send_request(
                    Request::builder()
                        .method(Method::GET)
                        .uri(format!("http://unix.socket/nolanv{}", i))
                        .body(Body::empty())
                        .expect("Request builder"),
                )
                .await
                .expect("Response");

            assert_eq!(response.status().as_str(), "200");

            assert_eq!(
                response_to_utf8(response).await,
                Some(format!("Hello nolanv{}", i))
            )
        }
    }

    #[tokio::test]
    async fn server_not_started() {
        let socket_path = make_socket_path_test("client", "server_not_started");

        let http_client = HttpClientUnixDomainSocket::<Body>::try_new(&socket_path).await;
        assert!(matches!(
            http_client.err(),
            Some(Error::SocketConnectionInitiation(_))
        ));
    }

    #[tokio::test]
    async fn server_stopped() {
        let (server, mut client) = make_client_server("server_stopped").await;
        server.abort().await;

        let response = client
            .send_request(
                Request::builder()
                    .method(Method::GET)
                    .uri("http://unix.socket/nolanv")
                    .body(Body::empty())
                    .expect("Request builder"),
            )
            .await;
        assert!(matches!(
            response.err(),
                         Some(Error::SendRequest(e)) if e.is_canceled()
        ));

        let _ = Server::try_new(&make_socket_path_test("client", "server_stopped"))
            .await
            .expect("Server::try_new");
        let mut http_client = client.try_reconnect().await.expect("client.try_reconnect");

        let response = http_client
            .send_request(
                Request::builder()
                    .method(Method::GET)
                    .uri("http://unix.socket/nolanv")
                    .body(Body::empty())
                    .expect("Request builder"),
            )
            .await
            .expect("Response");

        assert_eq!(response.status().as_str(), "200");

        assert_eq!(
            response_to_utf8(response).await,
            Some("Hello nolanv".into())
        )
    }
}
