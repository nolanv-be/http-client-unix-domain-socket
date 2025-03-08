use std::path::PathBuf;

use axum::{Router, extract::Path, routing::get};
use tokio::{
    fs::{create_dir_all, remove_file, try_exists},
    net::UnixListener,
    task::JoinHandle,
};

#[derive(Debug)]
#[allow(dead_code)]
pub enum ErrorServer {
    CheckOldSocketExist(std::io::Error),
    RemoveOldSocket(std::io::Error),
    SocketNoParentDir,
    CreateSocketParentDir(std::io::Error),
    SocketBind(std::io::Error),
    ServerHandleError,
    ServerHandleStopped,
}

#[derive(Debug)]
pub struct Server {
    server_handle: JoinHandle<ErrorServer>,
}

impl Server {
    pub async fn try_new(socket_path: &str) -> Result<Self, ErrorServer> {
        Server::try_listen(socket_path.into()).await
    }

    async fn try_listen(socket_path: PathBuf) -> Result<Self, ErrorServer> {
        let is_socket_exist = try_exists(socket_path.clone())
            .await
            .map_err(ErrorServer::CheckOldSocketExist)?;
        match is_socket_exist {
            true => {
                remove_file(socket_path.clone())
                    .await
                    .map_err(ErrorServer::RemoveOldSocket)?;
            }
            false => {
                create_dir_all(
                    socket_path
                        .clone()
                        .parent()
                        .ok_or(ErrorServer::SocketNoParentDir)?,
                )
                .await
                .map_err(ErrorServer::CreateSocketParentDir)?;
            }
        }

        let socket = UnixListener::bind(socket_path.clone()).map_err(ErrorServer::SocketBind)?;

        let server_handle = tokio::task::spawn(async move {
            let app = Router::new()
                .route("/{name}", get(Server::respond))
                .route("/json/{name}", get(Server::respond_json))
                .into_make_service();

            if axum::serve(socket, app).await.is_err() {
                return ErrorServer::ServerHandleError;
            }

            ErrorServer::ServerHandleStopped
        });

        Ok(Server { server_handle })
    }

    async fn respond(Path(name): Path<String>) -> String {
        format!("Hello {}", name)
    }

    async fn respond_json(Path(name): Path<String>) -> String {
        format!("{{\"hello\": \"{}\"}}", name)
    }

    pub async fn abort(self) -> Option<ErrorServer> {
        self.server_handle.abort();
        self.server_handle.await.ok()
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    async fn it_works() {
        let server =
            Server::try_new("/tmp/http_client_unix_domain_socket/server_test.socket").await;
        assert!(server.is_ok());
    }
}
