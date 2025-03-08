use crate::{Error, HttpClientUnixDomainSocket};
use axum::http::request::Builder;
use axum_core::body::Body;
use http_body_util::BodyExt;
use hyper::{Method, Request, StatusCode};
use serde::de::DeserializeOwned;
impl HttpClientUnixDomainSocket<Body> {
    pub async fn send_get<T: DeserializeOwned>(
        &mut self,
        headers: &[(&str, &str)],
        endpoint: &str,
    ) -> Result<(StatusCode, T), Error> {
        let request = HttpClientUnixDomainSocket::build_headers(Request::builder(), headers)
            .method(Method::GET)
            .uri(format!("http://unix.socket{}", endpoint))
            .body(Body::empty())
            .map_err(Error::RequestBuild)?;

        let response = self.send_request(request).await?;

        let is_success = response.status().is_success();
        let status_code = response.status();
        let body = response
            .collect()
            .await
            .map_err(Error::ResponseCollect)?
            .to_bytes();

        if !is_success {
            return Err(Error::ResponseUnsuccessful(
                status_code.as_str().into(),
                String::from_utf8(body.to_vec()).unwrap_or("Body not UTF8".into()),
            ));
        }

        Ok((
            status_code,
            serde_json::from_slice(&body.to_vec()).map_err(Error::ResponseParsing)?,
        ))
    }
    fn build_headers(request_builder: Builder, headers: &[(&str, &str)]) -> Builder {
        let mut request_builder = request_builder;
        for header in headers {
            request_builder = request_builder.header(header.0, header.1);
        }

        request_builder
    }
}
#[cfg(test)]
mod tests {
    use serde_json::{Value, json};

    use crate::test_helpers::util::make_client_server;

    #[tokio::test]
    async fn simple_get_request() {
        let (_, mut client) = make_client_server("simple_get_request").await;

        let response = client
            .send_get::<Value>(&[], "/json/nolanv")
            .await
            .expect("Response");

        assert_eq!(response.0.as_str(), "200");
        assert_eq!(response.1.get("hello"), Some(&json!("nolanv")))
    }
}
