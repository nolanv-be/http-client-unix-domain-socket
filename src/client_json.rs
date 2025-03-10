use crate::{
    ClientUnix, Error,
    error::{ErrorAndResponse, ErrorAndResponseJson},
};
use axum_core::body::Body;
use hyper::{Method, StatusCode};
use serde::{Serialize, de::DeserializeOwned};

pub type JsonClientUnix = ClientUnix;

impl ClientUnix {
    pub async fn send_request_json<IN: Serialize, OUT: DeserializeOwned, ERR: DeserializeOwned>(
        &mut self,
        endpoint: &str,
        method: Method,
        headers: &[(&str, &str)],
        body_request: Option<&IN>,
    ) -> Result<(StatusCode, OUT), ErrorAndResponseJson<ERR>> {
        let mut headers = headers.to_vec();
        headers.push(("Content-Type", "application/json"));

        let body_request = match body_request {
            Some(body_request) => Body::from(
                serde_json::to_vec(body_request)
                    .map_err(|e| ErrorAndResponseJson::InternalError(Error::RequestParsing(e)))?,
            ),
            None => Body::empty(),
        };

        match self
            .send_request(endpoint, method, &headers, Some(body_request))
            .await
        {
            Ok((status_code, response)) => Ok((
                status_code,
                serde_json::from_slice(&response).map_err(|e| {
                    ErrorAndResponseJson::InternalError(Error::ResponseParsing(e, response))
                })?,
            )),
            Err(ErrorAndResponse::InternalError(e)) => Err(ErrorAndResponseJson::InternalError(e)),
            Err(ErrorAndResponse::ResponseUnsuccessful(status_code, response)) => {
                Err(ErrorAndResponseJson::ResponseUnsuccessful(
                    status_code,
                    serde_json::from_slice(&response).map_err(|e| {
                        ErrorAndResponseJson::InternalError(Error::ResponseParsing(e, response))
                    })?,
                ))
            }
        }
    }
}
#[cfg(test)]
mod tests {
    use hyper::{Method, StatusCode};
    use serde::{Deserialize, Serialize};
    use serde_json::{Value, json};

    use crate::{error::ErrorAndResponseJson, test_helpers::util::make_client_server};

    #[derive(Deserialize, Debug)]
    struct ErrorJson {
        msg: String,
    }

    #[tokio::test]
    async fn simple_get_request() {
        let (_, mut client) = make_client_server("simple_get_request").await;

        let response = client
            .send_request_json::<(), Value, Value>("/json/nolanv", Method::GET, &[], None)
            .await
            .expect("client.send_request_json");

        assert_eq!(response.0, StatusCode::OK);
        assert_eq!(response.1.get("hello"), Some(&json!("nolanv")))
    }

    #[tokio::test]
    async fn simple_get_404_request() {
        let (_, mut client) = make_client_server("simple_get_404_request").await;

        let result = client
            .send_request_json::<(), Value, ErrorJson>("/json/nolanv/nop", Method::GET, &[], None)
            .await;

        dbg!(&result);
        assert!(matches!(
            result.err(),
            Some(ErrorAndResponseJson::ResponseUnsuccessful(status_code, body))
                if status_code == StatusCode::NOT_FOUND && body.msg == "not found"
        ));
    }

    #[tokio::test]
    async fn simple_post_request() {
        let (_, mut client) = make_client_server("simple_post_request").await;

        #[derive(Serialize)]
        struct NameJson {
            name: String,
        }

        let response = client
            .send_request_json::<NameJson, Value, Value>(
                "/json",
                Method::POST,
                &[],
                Some(&NameJson {
                    name: "nolanv".into(),
                }),
            )
            .await
            .expect("client.send_request_json");

        assert_eq!(response.0, StatusCode::OK);
        assert_eq!(response.1.get("hello"), Some(&json!("nolanv")))
    }

    #[tokio::test]
    async fn simple_post_bad_request() {
        let (_, mut client) = make_client_server("simple_post_bad_request").await;

        #[derive(Serialize)]
        struct NameBadJson {
            nom: String,
        }

        let result = client
            .send_request_json::<NameBadJson, Value, ErrorJson>(
                "/json",
                Method::POST,
                &[],
                Some(&NameBadJson {
                    nom: "nolanv".into(),
                }),
            )
            .await;

        assert!(matches!(
            result.err(),
            Some(ErrorAndResponseJson::ResponseUnsuccessful(status_code, body))
                if status_code == StatusCode::BAD_REQUEST && body.msg == "bad request"
        ));
    }
}
