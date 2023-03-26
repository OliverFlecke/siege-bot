use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use uuid::Uuid;

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

    pub async fn connect(&self) -> Result<ConnectResponse, Box<dyn Error>> {
        let client = reqwest::Client::new();
        let response = client
            .post("https://public-ubiservices.ubi.com/v3/profiles/sessions")
            .header("Content-Type", "application/json; charset-UTF-8")
            .header("Ubi-AppId", "39baebad-39e5-4552-8c25-2c9b919064e2")
            .header("Authorization", format!("Basic {}", self.get_token()))
            .send()
            .await?;

        // println!("{}", response.text().await?);
        // todo!()
        Ok(response.json::<ConnectResponse>().await?)
    }

    pub fn from_environment() -> Self {
        Self {
            username: std::env::var("UBISOFT_EMAIL")
                .expect("Variable `UBISOFT_EMAIL` be set as an environment variable"),
            password: std::env::var("UBISOFT_PASSWORD")
                .expect("Variable `UBISOFT_PASSWORD` be set as an environment variable"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PlatformType {
    Uplay,
    // These have not been verified
    Xbox,
    Playstation,
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
    async fn connect() {
        let auth = Auth::from_environment();

        let response = auth.connect().await;
        println!("{:?}", response.unwrap());
    }
}
