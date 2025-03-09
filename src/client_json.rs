use crate::{Error, HttpClientUnixDomainSocket};
use axum_core::body::Body;
use http_body_util::BodyExt;
use hyper::{Method, Request, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

impl HttpClientUnixDomainSocket<Body> {
    pub async fn send_request_json<IN: Serialize, OUT: DeserializeOwned>(
        &mut self,
        endpoint: &str,
        method: Method,
        headers: &[(&str, &str)],
        body_request: Option<&IN>,
    ) -> Result<(StatusCode, OUT), Error> {
        let body_request = match body_request {
            Some(body_request) => {
                Body::from(serde_json::to_vec(body_request).map_err(Error::BodyParsing)?)
            }
            None => Body::empty(),
        };
        let mut request_builder = Request::builder();
        for header in headers {
            request_builder = request_builder.header(header.0, header.1);
        }

        let request = request_builder
            .header("Content-Type", "application/json")
            .method(method)
            .uri(format!("http://unix.socket{}", endpoint))
            .body(body_request)
            .map_err(Error::RequestBuild)?;

        let response = self.send_request(request).await?;

        let is_success = response.status().is_success();
        let status_code = response.status();
        let body_response = response
            .collect()
            .await
            .map_err(Error::ResponseCollect)?
            .to_bytes();

        if !is_success {
            return Err(Error::ResponseUnsuccessful(
                status_code.as_str().into(),
                String::from_utf8(body_response.to_vec()).unwrap_or("Body not UTF8".into()),
            ));
        }

        Ok((
            status_code,
            serde_json::from_slice(&body_response).map_err(Error::ResponseParsing)?,
        ))
    }
}
#[cfg(test)]
mod tests {
    use hyper::Method;
    use serde::Serialize;
    use serde_json::{Value, json};

    use crate::test_helpers::util::make_client_server;

    #[tokio::test]
    async fn simple_get_request() {
        let (_, mut client) = make_client_server("simple_get_request").await;

        let response = client
            .send_request_json::<(), Value>("/json/nolanv", Method::GET, &[], None)
            .await
            .expect("Response");

        assert_eq!(response.0.as_str(), "200");
        assert_eq!(response.1.get("hello"), Some(&json!("nolanv")))
    }

    #[tokio::test]
    async fn simple_post_request() {
        let (_, mut client) = make_client_server("simple_post_request").await;

        #[derive(Serialize)]
        struct NameJson {
            name: String,
        }

        let response = client
            .send_request_json::<NameJson, Value>(
                "/json",
                Method::POST,
                &[],
                Some(&NameJson {
                    name: "nolanv".into(),
                }),
            )
            .await
            .expect("Response");

        assert_eq!(response.0.as_str(), "200");
        assert_eq!(response.1.get("hello"), Some(&json!("nolanv")))
    }
}
