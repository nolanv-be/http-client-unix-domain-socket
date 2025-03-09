use crate::{ClientUnix, test_helpers::server::Server};
use axum_core::body::Body;
use http_body_util::BodyExt;

use hyper::{Response, body::Incoming};

pub fn make_socket_path_test(test_file: &str, test_function: &str) -> String {
    format!(
        "/tmp/http_client_unix_domain_socket/{}_{}.socket",
        test_file, test_function
    )
}

pub async fn response_to_utf8(response: Response<Incoming>) -> Option<String> {
    let response = response.collect().await.ok()?;

    String::from_utf8(response.to_bytes().to_vec()).ok()
}

pub async fn make_client_server(test_function: &str) -> (Server, ClientUnix<Body>) {
    let socket_path = make_socket_path_test("client", test_function);
    let server = Server::try_new(&socket_path)
        .await
        .expect("Server::try_new");
    let client = ClientUnix::<Body>::try_new(&socket_path)
        .await
        .expect("ClientUnix::try_new");

    (server, client)
}
