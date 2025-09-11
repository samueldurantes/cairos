use reqwest::StatusCode;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Request failed (Status: {0:?}) = {1:?}")]
    Request(Option<StatusCode>, String),
    #[error("Failed to deserialize")]
    Deserialization,
}

pub mod cairos {
    use super::Error;
    use reqwest::{
        StatusCode,
        header::{ACCEPT, AUTHORIZATION},
    };
    use serde::{Deserialize, Serialize};

    #[derive(Serialize)]
    pub struct LoginParams {
        pub access_token: String,
    }

    #[derive(Deserialize)]
    pub struct LoginResponse {
        pub token: String,
    }

    pub async fn login(
        reqwest: &reqwest::Client,
        base_url: &str,
        p: LoginParams,
    ) -> Result<LoginResponse, Error> {
        let result = reqwest
            .post(format!("{base_url}/auth/login"))
            .header(ACCEPT, "application/json")
            .json(&p)
            .send()
            .await;

        match result {
            Ok(response) => response
                .json::<LoginResponse>()
                .await
                .map_err(|_| Error::Deserialization),
            Err(error) => Err(Error::Request(error.status(), error.to_string())),
        }
    }

    #[derive(Serialize)]
    pub struct SendEventsParams {
        pub uri: String,
        pub is_write: bool,
        pub language: Option<String>,
        pub line_number: Option<i32>,
        pub cursor_pos: Option<i32>,
    }

    pub async fn send_events(
        reqwest: &reqwest::Client,
        base_url: &str,
        api_token: &str,
        p: SendEventsParams,
    ) -> Result<(), Error> {
        let result = reqwest
            .post(format!("{base_url}/events/capture"))
            .header(ACCEPT, "application/json")
            .header(AUTHORIZATION, format!("Bearer {api_token}"))
            .json(&p)
            .send()
            .await;

        match result {
            Ok(response) => {
                let status = response.status();
                if status != StatusCode::OK {
                    let text = response.text().await.unwrap_or(String::new());
                    return Err(Error::Request(Some(status), text));
                }

                Ok(())
            }
            Err(error) => Err(Error::Request(error.status(), error.to_string())),
        }
    }
}

pub mod github {
    use super::Error;
    use reqwest::header::ACCEPT;
    use serde::Deserialize;

    const GITHUB_CLIENT_ID: &str = "Ov23lifzTXvNg6MaDMm8";

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
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", GITHUB_CLIENT_ID),
                ("scope", "read:user, user:email"),
            ])
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
            .header(ACCEPT, "application/json")
            .form(&[
                ("client_id", GITHUB_CLIENT_ID),
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
