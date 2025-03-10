use crate::{ClientUnix, test_helpers::server::Server};

pub fn make_socket_path_test(test_file: &str, test_function: &str) -> String {
    format!(
        "/tmp/http_client_unix_domain_socket/{}_{}.socket",
        test_file, test_function
    )
}

pub async fn make_client_server(test_function: &str) -> (Server, ClientUnix) {
    let socket_path = make_socket_path_test("client", test_function);
    let server = Server::try_new(&socket_path)
        .await
        .expect("Server::try_new");
    let client = ClientUnix::try_new(&socket_path)
        .await
        .expect("ClientUnix::try_new");

    (server, client)
}
