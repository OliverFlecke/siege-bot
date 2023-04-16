use chrono::{Months, NaiveDate, Utc};
use reqwest::{RequestBuilder, Url};
use serde::Deserialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::{Auth, ConnectError, ConnectResponse};
use crate::constants::{UBI_APP_ID, UBI_USER_AGENT};
use crate::models::{
    FullProfile, PlatformType, PlayerProfile, PlaytimeProfile, PlaytimeResponse, RankedV2Response,
    StatisticResponse,
};

#[derive(Debug)]
pub struct Client {
    auth: RwLock<ConnectResponse>,
    client: reqwest::Client,
}

impl From<ConnectResponse> for Client {
    fn from(auth: ConnectResponse) -> Self {
        Self {
            auth: RwLock::new(auth),
            client: reqwest::Client::new(),
        }
    }
}

impl Client {
    /// Search for a Ubisoft player ID.
    pub async fn search_for_player(&self, name: &str) -> Result<Uuid, ConnectError> {
        #[derive(Deserialize)]
        struct Response {
            profiles: Vec<PlayerProfile>,
        }

        let url = "https://public-ubiservices.ubi.com/v3/profiles";
        let url = Url::parse_with_params(
            url,
            &[
                ("nameOnPlatform", name),
                ("platformType", PlatformType::Uplay.to_string().as_str()),
            ],
        )
        .expect("should always be a valid url");

        let response = self.get(url).await?;
        let profile: Response = response
            .json()
            .await
            .map_err(|_| ConnectError::UnexpectedResponse)?;

        Ok(*profile.profiles[0].profile_id())
    }

    /// Get the playtime for the given player.
    /// See the `PlaytimeProfile` structs for the fields it contains.
    pub async fn get_playtime(&self, player_id: Uuid) -> Result<PlaytimeProfile, ConnectError> {
        // TODO: Should `spaceId` be a parameter?
        let url = "https://public-ubiservices.ubi.com/v1/profiles/stats";
        let url = Url::parse_with_params(
            url,
            &[
                ("profileIds", player_id.to_string()),
                (
                    "spaceId",
                    "0d2ae42d-4c27-4cb7-af6c-2099062302bb".to_string(),
                ),
                (
                    "statsName",
                    "PPvPTimePlayed,PPvETimePlayed,PTotalTimePlayed,PClearanceLevel".to_string(),
                ),
            ],
        )
        .expect("url is valid");

        let response = self.get(url).await?;
        let parsed = response
            .json::<PlaytimeResponse>()
            .await
            .map_err(|_| ConnectError::UnexpectedResponse)?;

        Ok(parsed.profiles()[0])
    }

    /// Get full Siege profiles from the API. This will only contain the latest
    /// statistics from the current season. It does *not* look like it is possible
    /// to query earlier seasons at the moment.
    pub async fn get_full_profiles(
        &self,
        player_id: Uuid,
    ) -> Result<Vec<FullProfile>, ConnectError> {
        let url = "https://public-ubiservices.ubi.com/v2/spaces/0d2ae42d-4c27-4cb7-af6c-2099062302bb/title/r6s/skill/full_profiles";
        let url = Url::parse_with_params(
            url,
            &[
                ("profile_ids", player_id.to_string()),
                // TODO: This mapping should happen from `PlatformType`
                ("platform_families", "pc".to_string()), // platform.to_string().to_lowercase()),
            ],
        )
        .expect("url is valid");

        let response = self.get(url).await?;
        let profile = response.json::<RankedV2Response>().await.map_err(|err| {
            tracing::error!("Error: {err:?}");
            ConnectError::UnexpectedResponse
        })?;

        Ok(profile.platform_families_full_profiles()[0]
            .board_ids_full_profiles()
            .iter()
            .map(|x| x.full_profiles()[0])
            .collect::<Vec<_>>())
    }

    /// Retreive statistics about operators for a given player.
    pub async fn get_operators(&self, player_id: Uuid) -> Result<StatisticResponse, ConnectError> {
        let url = create_summary_query(player_id, AggregationType::Operators);
        let response = self.get(url).await?;
        response
            .json::<StatisticResponse>()
            .await
            .map_err(|_| ConnectError::UnexpectedResponse)
    }

    /// Get maps statistics for a given player.
    pub async fn get_maps(&self, player_id: Uuid) -> Result<StatisticResponse, ConnectError> {
        let url = create_summary_query(player_id, AggregationType::Maps);
        let response = self.get(url).await?;

        response.json::<StatisticResponse>().await.map_err(|err| {
            tracing::error!("Error: {err:?}");
            ConnectError::UnexpectedResponse
        })
    }

    async fn get(&self, url: Url) -> Result<reqwest::Response, ConnectError> {
        if self.auth.read().await.is_expired() {
            self.refresh_auth().await?;
        }

        self.client
            .get(url)
            .set_headers(&*self.auth.read().await)
            .send()
            .await
            .map_err(ConnectError::ConnectionError)
    }

    /// Refresh the authentication session to Ubisoft's API.
    async fn refresh_auth(&self) -> Result<(), ConnectError> {
        tracing::info!("Refreshing auth token for client");
        // TODO: Would prefer not reading this from the environment again.
        // They could be set in another way in the future.
        let auth = Auth::from_environment().connect().await?;

        *self.auth.write().await = auth;

        Ok(())
    }
}

trait SetHeaders {
    fn set_headers(self, auth: &ConnectResponse) -> Self;
}

impl SetHeaders for RequestBuilder {
    fn set_headers(self, auth: &ConnectResponse) -> Self {
        self.header("User-Agent", UBI_USER_AGENT)
            .header("Ubi-AppId", UBI_APP_ID)
            .header("Ubi-LocalCode", "en-US")
            .header("Ubi-SessionId", auth.session_id().to_string())
            .header("Authorization", format!("Ubi_v1 t={}", auth.ticket()))
            .header("Connection", "keep-alive")
            .header(
                "expiration",
                auth.expiration()
                    .to_rfc3339_opts(chrono::SecondsFormat::Micros, true),
            )
    }
}

#[derive(Debug, strum::Display)]
pub enum AggregationType {
    Operators,
    Summary,
    Maps,
}

fn create_summary_query(player_id: Uuid, aggregation: AggregationType) -> Url {
    #[allow(dead_code)]
    fn format_date(date: NaiveDate) -> String {
        date.format("%Y%m%d").to_string()
    }

    // NOTE: seems like these cannot be more than three (somthing is off with my math) months apart.
    // Or maybe it just cannot go earlier than 2022-11-25. Not sure what is special about that date.
    // let start_date = NaiveDate::from_ymd_opt(2022, 11, 25).expect("is a valid date"); // TODO: Find a proper default
    let end_date = Utc::now().date_naive();
    #[allow(unused_variables)]
    let start_date = end_date
        .checked_sub_months(Months::new(3))
        .expect("should be valid");

    let url = format!("https://prod.datadev.ubisoft.com/v1/profiles/{player_id}/playerstats");
    Url::parse_with_params(
        url.as_str(),
        &[
            ("view", "current"),
            ("platformGroup", "PC"),
            (
                "aggregation",
                aggregation.to_string().to_lowercase().as_str(),
            ),
            (
                "spaceId",
                PlatformType::Uplay.get_space().to_string().as_str(),
            ),
            (
                "gameMode",
                vec!["all", "ranked", "cansal", "unranked"]
                    .join(",")
                    .as_str(),
            ),
            (
                "teamRole",
                vec!["all", "Attacker", "Defender"].join(",").as_str(),
            ),
            // ("startDate", format_date(start_date).as_str()),
            // ("endDate", format_date(end_date).as_str()),
        ],
    )
    .expect("is a valid url")
}

#[cfg(test)]
mod test {
    use async_once::AsyncOnce;
    use chrono::{DateTime, Duration};
    use lazy_static::lazy_static;

    use crate::auth::Auth;

    use super::*;

    lazy_static! {
        static ref CONNECTED_AUTH: AsyncOnce<ConnectResponse> =
            AsyncOnce::new(async { Auth::from_environment().connect().await.unwrap() });
    }

    async fn get_client() -> Client {
        Into::<Client>::into(CONNECTED_AUTH.get().await.clone())
    }

    fn mock_player_id() -> Uuid {
        Uuid::parse_str("e7679633-31ff-4f44-8cfd-d0ff81e2c10a").expect("this is a valid guid")
    }

    #[test]
    fn operators_url() {
        let expected = "https://prod.datadev.ubisoft.com/v1/profiles/e7679633-31ff-4f44-8cfd-d0ff81e2c10a/playerstats?view=current&platformGroup=PC&aggregation=operators&spaceId=5172a557-50b5-4665-b7db-e3f2e8c5041d&gameMode=all%2Cranked%2Ccansal%2Cunranked&teamRole=all%2CAttacker%2CDefender";

        let actual = create_summary_query(mock_player_id(), AggregationType::Operators);
        assert_eq!(actual.as_str(), expected);
    }

    #[tokio::test]
    async fn search_player() {
        let id = get_client()
            .await
            .search_for_player("NaoFredzibob")
            .await
            .unwrap();
        assert_eq!(
            id,
            Uuid::parse_str("e7679633-31ff-4f44-8cfd-d0ff81e2c10a").expect("is valid")
        )
    }

    #[tokio::test]
    async fn operators_statistics() {
        let stats = get_client()
            .await
            .get_operators(mock_player_id())
            .await
            .unwrap();
        println!("{:?}", stats);
    }

    #[tokio::test]
    async fn maps_statistics() {
        let stats = get_client().await.get_maps(mock_player_id()).await.unwrap();
        println!("{:?}", stats);
    }

    #[tokio::test]
    async fn full_player_profiles() {
        _ = get_client()
            .await
            .get_full_profiles(mock_player_id())
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn playtime() {
        let player_id = mock_player_id();

        let playtime = get_client().await.get_playtime(player_id).await.unwrap();

        // Assert PvP
        assert!(*playtime.statistics().pvp_time_played().duration() > Duration::hours(1));
        assert!(
            *playtime.statistics().pvp_time_played().start_date()
                == DateTime::parse_from_rfc3339("2021-08-30T11:10:00.200Z").unwrap()
        );
        assert!(
            *playtime.statistics().pvp_time_played().last_modified()
                > *playtime.statistics().pvp_time_played().start_date()
        );

        // Assert PvE
        assert!(*playtime.statistics().pve_time_played().duration() > Duration::hours(1));
        assert!(
            *playtime.statistics().pve_time_played().start_date()
                == DateTime::parse_from_rfc3339("2021-08-30T11:08:00.415Z").unwrap()
        );
        assert!(
            *playtime.statistics().pve_time_played().last_modified()
                > *playtime.statistics().pve_time_played().start_date()
        );

        // Assert total time played
        assert!(*playtime.statistics().total_time_played().duration() > Duration::hours(1));
        assert!(
            *playtime.statistics().total_time_played().start_date()
                == DateTime::parse_from_rfc3339("2021-08-30T11:13:00.398Z").unwrap()
        );
        assert!(
            *playtime.statistics().total_time_played().last_modified()
                > *playtime.statistics().total_time_played().start_date()
        );

        // Assert clearance level
        assert!(*playtime.statistics().clearance_level().duration() >= Duration::seconds(123));
        assert!(
            *playtime.statistics().clearance_level().start_date()
                == DateTime::parse_from_rfc3339("2021-08-30T11:15:00.426Z").unwrap()
        );
        assert!(
            *playtime.statistics().clearance_level().last_modified()
                > *playtime.statistics().clearance_level().start_date()
        );
    }
}
