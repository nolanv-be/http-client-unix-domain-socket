use hyper::StatusCode;
#[cfg(feature = "json")]
use serde::de::DeserializeOwned;

#[derive(Debug)]
pub enum Error {
    SocketConnectionInitiation(std::io::Error),
    SocketConnectionClosed(Option<hyper::Error>),
    Handhsake(hyper::Error),
    RequestBuild(hyper::http::Error),
    RequestSend(hyper::Error),
    #[cfg(feature = "json")]
    RequestParsing(serde_json::Error),
    ResponseCollect(hyper::Error),
    #[cfg(feature = "json")]
    ResponseParsing(serde_json::Error),
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::SocketConnectionInitiation(e) => {
                write!(f, "Failed to connect to unix stream, {}", e)
            }
            Error::SocketConnectionClosed(None) => {
                write!(f, "Unix stream was closed without any error.")
            }
            Error::SocketConnectionClosed(Some(e)) => {
                write!(f, "Unix stream was closed, {}", e)
            }
            Error::Handhsake(e) => {
                write!(f, "Failed to do HTTP 1.0 handshaking, {}", e)
            }
            Error::RequestBuild(e) => {
                write!(f, "Failed to build http request, {}", e)
            }
            Error::RequestSend(e) => {
                write!(f, "Failed to send http request, {}", e)
            }
            #[cfg(feature = "json")]
            Error::RequestParsing(e) => {
                write!(f, "Failed to parse http json request, {}", e)
            }
            Error::ResponseCollect(e) => {
                write!(f, "Failed to collect http request, {}", e)
            }
            #[cfg(feature = "json")]
            Error::ResponseParsing(e) => {
                write!(f, "Failed to parse http json response, {}", e)
            }
        }
    }
}

#[derive(Debug)]
pub enum ErrorAndResponse {
    InternalError(Error),
    ResponseUnsuccessful(StatusCode, Vec<u8>),
}
impl std::fmt::Display for ErrorAndResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorAndResponse::InternalError(e) => {
                write!(f, "Internal error, {}", e)
            }
            ErrorAndResponse::ResponseUnsuccessful(status_code, _) => {
                write!(
                    f,
                    "HTTP response was not successful, status code = {}",
                    status_code
                )
            }
        }
    }
}

#[cfg(feature = "json")]
#[derive(Debug)]
pub enum ErrorAndResponseJson<ERR: DeserializeOwned> {
    InternalError(Error),
    ResponseUnsuccessful(StatusCode, ERR),
}
#[cfg(feature = "json")]
impl<ERR: DeserializeOwned> std::fmt::Display for ErrorAndResponseJson<ERR> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            ErrorAndResponseJson::InternalError(e) => {
                write!(f, "Internal error, {}", e)
            }
            ErrorAndResponseJson::ResponseUnsuccessful(status_code, _) => {
                write!(
                    f,
                    "HTTP response was not successful, status code = {}",
                    status_code
                )
            }
        }
    }
}
