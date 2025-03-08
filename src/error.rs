#[derive(Debug)]
pub enum Error {
    SocketConnectionInitiation(std::io::Error),
    SocketConnectionClosed(Option<hyper::Error>),
    Handhsake(hyper::Error),
    SendRequest(hyper::Error),
    RequestBuild(hyper::http::Error),
    ResponseCollect(hyper::Error),
    ResponseUnsuccessful(String, String),
    #[cfg(feature = "json")]
    ResponseParsing(serde_json::Error),
    #[cfg(feature = "json")]
    BodyParsing(serde_json::Error),
}
