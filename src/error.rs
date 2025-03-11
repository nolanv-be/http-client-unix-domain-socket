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
impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::SocketConnectionInitiation(error) => Some(error),
            Error::SocketConnectionClosed(Some(error)) => Some(error),
            Error::SocketConnectionClosed(None) => None,
            Error::Handhsake(error) => Some(error),
            Error::RequestBuild(error) => Some(error),
            Error::RequestSend(error) => Some(error),
            #[cfg(feature = "json")]
            Error::RequestParsing(error) => Some(error),
            Error::ResponseCollect(error) => Some(error),
            #[cfg(feature = "json")]
            Error::ResponseParsing(error) => Some(error),
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
impl std::error::Error for ErrorAndResponse {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorAndResponse::InternalError(error) => error.source(),
            ErrorAndResponse::ResponseUnsuccessful(_, _) => None,
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
#[cfg(feature = "json")]
impl<ERR: DeserializeOwned + std::fmt::Debug> std::error::Error for ErrorAndResponseJson<ERR> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ErrorAndResponseJson::InternalError(error) => error.source(),
            ErrorAndResponseJson::ResponseUnsuccessful(_, _) => None,
        }
    }
}
