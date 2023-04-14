use chrono::prelude::*;
use derive_getters::Getters;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{constants::UBI_APP_ID, models::PlatformType};

#[derive(Debug)]
pub struct Auth {
    username: String,
    password: String,
}

impl Auth {
    pub fn get_token(&self) -> String {
        use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

        URL_SAFE_NO_PAD.encode(format!(
            "{username}:{password}",
            username = self.username,
            password = self.password
        ))
    }

    pub async fn connect(&self) -> Result<ConnectResponse, ConnectError> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
            .header("Content-Type", "application/json; charset-UTF-8")
            .header("Ubi-AppId", UBI_APP_ID)
            .header("Authorization", format!("Basic {}", self.get_token()))
            .send()
            .await
            .map_err(ConnectError::ConnectionError)?;

        if response.status().is_success() {
            Ok(response
                .json::<ConnectResponse>()
                .await
                .map_err(|_| ConnectError::UnexpectedResponse)?)
        } else {
            println!(
                "{}",
                response
                    .text()
                    .await
                    .map_err(|_| ConnectError::UnexpectedResponse)?
            );
            Err(ConnectError::InvalidPassword)
        }
    }

    /// Create a new Auth context with a username and password.
    pub fn new(username: String, password: String) -> Self {
        Self { username, password }
    }

    /// Load Auth from the environment. This expects the `UBISOFT_EMAIL` and
    /// `UBISOFT_PASSWORD` variables to be set. Otherwise this will panic.
    pub fn from_environment() -> Self {
        Self {
            username: std::env::var("UBISOFT_EMAIL")
                .expect("Variable `UBISOFT_EMAIL` be set as an environment variable"),
            password: std::env::var("UBISOFT_PASSWORD")
                .expect("Variable `UBISOFT_PASSWORD` be set as an environment variable"),
        }
    }
}

#[derive(Debug)]
pub enum ConnectError {
    InvalidPassword,
    UnexpectedResponse,
    ConnectionError(reqwest::Error),
}

impl PartialEq for ConnectError {
    fn eq(&self, other: &Self) -> bool {
        use ConnectError::*;
        matches!(
            (self, other),
            (InvalidPassword, InvalidPassword)
                | (UnexpectedResponse, UnexpectedResponse)
                | (ConnectionError(_), ConnectionError(_))
        )
    }
}

#[derive(Debug, Deserialize, Serialize, Getters)]
#[serde(rename_all = "camelCase")]
pub struct ConnectResponse {
    platform_type: PlatformType,
    ticket: String, // Base64 encoded
    profile_id: Uuid,
    user_id: Uuid,
    name_on_platform: String,
    environment: String, // NOTE: Might convert to enum
    expiration: DateTime<Utc>,
    space_id: Uuid,
    // client_ip: String,
    // client_ip_country: String,
    server_time: DateTime<Utc>,
    session_id: Uuid,
    session_key: String, // Base64 encoded
}

impl ConnectResponse {
    /// Check if the session is expired.
    pub fn is_expired(&self) -> bool {
        self.expiration < Utc::now()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    fn mock_auth() -> Auth {
        Auth {
            username: "jomahebam.redafapap@rungel.net".to_string(),
            password: "4pVo9!9^D8BU4zet".to_string(),
        }
    }

    #[test]
    fn auth_token() {
        let auth = mock_auth();
        assert_eq!(
            auth.get_token(),
            "am9tYWhlYmFtLnJlZGFmYXBhcEBydW5nZWwubmV0OjRwVm85ITleRDhCVTR6ZXQ"
        );
    }

    #[tokio::test]
    async fn connect_with_incorrect_credentials() {
        let auth = Auth::new("abc".to_string(), "123".to_string());

        assert_eq!(
            auth.connect().await.unwrap_err(),
            ConnectError::InvalidPassword
        );
    }

    #[test]
    fn auth_debug() {
        let auth = Auth::new("abc".to_string(), "123".to_string());

        assert_eq!(
            format!("{auth:?}"),
            "Auth { username: \"abc\", password: \"123\" }"
        );
    }

    #[test]
    fn connect_error_eq() {
        assert_eq!(ConnectError::InvalidPassword, ConnectError::InvalidPassword);
        assert_eq!(
            ConnectError::UnexpectedResponse,
            ConnectError::UnexpectedResponse
        );

        // Not equal
        assert_ne!(
            ConnectError::InvalidPassword,
            ConnectError::UnexpectedResponse
        );
    }
}
