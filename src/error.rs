use hyper::StatusCode;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum Error {
    SocketConnectionInitiation(std::io::Error),
    SocketConnectionClosed(Option<hyper::Error>),
    Handhsake(hyper::Error),
    RequestSend(hyper::Error),
    RequestBuild(hyper::http::Error),
    #[cfg(feature = "json")]
    RequestParsing(serde_json::Error),
    ResponseCollect(hyper::Error),
    #[cfg(feature = "json")]
    ResponseParsing(serde_json::Error, Vec<u8>),
}

#[derive(Debug)]
pub enum ErrorAndResponse {
    InternalError(Error),
    ResponseUnsuccessful(StatusCode, Vec<u8>),
}

#[cfg(feature = "json")]
#[derive(Debug)]
pub enum ErrorAndResponseJson<ERR: DeserializeOwned> {
    InternalError(Error),
    ResponseUnsuccessful(StatusCode, ERR),
}
