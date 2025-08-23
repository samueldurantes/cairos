use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Request failed (Status: {0:?}) = {1:?}")]
    Request(Option<StatusCode>, String),
    #[error("Failed to deserialize")]
    Deserialization,
}

pub mod github {
    use super::Error;
    use reqwest::header::{ACCEPT, USER_AGENT};
    use serde::Deserialize;

    const CLIENT_ID: &str = "Ov23lifzTXvNg6MaDMm8";

    #[derive(Deserialize)]
    pub struct CreateUserCodesResponse {
        pub device_code: String,
        pub user_code: String,
        pub verification_uri: String,
        pub expires_in: i64,
        pub interval: u64,
    }

    pub async fn create_user_codes(
        reqwest: &reqwest::Client,
    ) -> Result<CreateUserCodesResponse, Error> {
        let result = reqwest
            .post("https://github.com/login/device/code")
            .header(USER_AGENT, "cairos-cli")
            .header(ACCEPT, "application/json")
            .form(&[("client_id", CLIENT_ID), ("scope", "read:user, user:email")])
            .send()
            .await;

        match result {
            Ok(response) => response
                .json::<CreateUserCodesResponse>()
                .await
                .map_err(|_| Error::Deserialization),
            Err(error) => Err(Error::Request(error.status(), error.to_string())),
        }
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    pub enum GetUserAuthorizedResponse {
        Success {
            access_token: String,
        },
        Error {
            #[serde(rename = "error")]
            _error: String,
        },
    }

    pub async fn get_user_authorized(
        reqwest: &reqwest::Client,
        device_code: &str,
    ) -> Result<GetUserAuthorizedResponse, Error> {
        let result = reqwest
            .post("https://github.com/login/oauth/access_token")
            .header(USER_AGENT, "cairos-cli")
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", CLIENT_ID),
                ("device_code", device_code),
                ("grant_type", "urn:ietf:params:oauth:grant-type:device_code"),
            ])
            .send()
            .await;

        match result {
            Ok(response) => response
                .json::<GetUserAuthorizedResponse>()
                .await
                .map_err(|_| Error::Deserialization),
            Err(error) => Err(Error::Request(error.status(), error.to_string())),
        }
    }
}
