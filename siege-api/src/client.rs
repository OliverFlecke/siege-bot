use async_trait::async_trait;
use chrono::{Months, NaiveDate, Utc};
use reqwest::{RequestBuilder, Url};
use serde::Deserialize;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::auth::{Auth, ConnectError, ConnectResponse};
use crate::constants::{
    DEFAULT_SPACE_ID, UBI_APP_ID, UBI_GAME_STATUS_URL, UBI_SERVICES_URL, UBI_USER_AGENT,
};
use crate::models::meta::GameStatus;
use crate::models::{
    PlatformType, PlayerProfile, PlaytimeProfile, PlaytimeResponse, RankedV2Response,
    StatisticResponse,
};

pub type Result<T> = core::result::Result<T, ConnectError>;

#[cfg_attr(test, mockall::automock)]
#[async_trait]
pub trait SiegeClient: Sync + Send {
    async fn search_for_player(&self, name: &str) -> Result<Uuid>;

    async fn get_playtime(&self, player_id: Uuid) -> Result<PlaytimeProfile>;

    async fn get_full_profiles(&self, player_id: Uuid) -> Result<RankedV2Response>;

    async fn get_operators(&self, player_id: Uuid) -> Result<StatisticResponse>;

    async fn get_maps(&self, player_id: Uuid) -> Result<StatisticResponse>;

    /// Get the current status of Siege's servers.
    async fn siege_status(&self) -> Result<Vec<GameStatus>>;
}

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

#[async_trait]
impl SiegeClient for Client {
    /// Search for a Ubisoft player ID.
    async fn search_for_player(&self, name: &str) -> Result<Uuid> {
        #[derive(Deserialize)]
        struct Response {
            profiles: Vec<PlayerProfile>,
        }

        let url = Url::parse_with_params(
            format!("{UBI_SERVICES_URL}/v3/profiles").as_str(),
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
    async fn get_playtime(&self, player_id: Uuid) -> Result<PlaytimeProfile> {
        let url = Url::parse_with_params(
            format!("{UBI_SERVICES_URL}/v1/profiles/stats").as_str(),
            &[
                ("profileIds", player_id.to_string()),
                ("spaceId", DEFAULT_SPACE_ID.to_string()),
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
    async fn get_full_profiles(&self, player_id: Uuid) -> Result<RankedV2Response> {
        let url = Url::parse_with_params(
            format!(
                "{UBI_SERVICES_URL}/v2/spaces/{DEFAULT_SPACE_ID}/title/r6s/skill/full_profiles"
            )
            .as_str(),
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

        Ok(profile)
    }

    /// Retreive statistics about operators for a given player.
    async fn get_operators(&self, player_id: Uuid) -> Result<StatisticResponse> {
        let url = create_summary_query(player_id, AggregationType::Operators);
        let response = self.get(url).await?;
        response.json::<StatisticResponse>().await.map_err(|err| {
            tracing::error!("Error: {err:?}");
            ConnectError::UnexpectedResponse
        })
    }

    /// Get maps statistics for a given player.
    async fn get_maps(&self, player_id: Uuid) -> Result<StatisticResponse> {
        let url = create_summary_query(player_id, AggregationType::Maps);
        let response = self.get(url).await?;

        response.json::<StatisticResponse>().await.map_err(|err| {
            tracing::error!("Error: {err:?}");
            ConnectError::UnexpectedResponse
        })
    }

    async fn siege_status(&self) -> Result<Vec<GameStatus>> {
        reqwest::get(UBI_GAME_STATUS_URL)
            .await
            .map_err(ConnectError::ConnectionError)?
            .json::<Vec<GameStatus>>()
            .await
            .map_err(|err| {
                println!("{err:?}");
                tracing::error!("Error: {err:?}");
                ConnectError::UnexpectedResponse
            })
            .map(|s| {
                s.into_iter()
                    .filter(|x| x.name().starts_with("Rainbow Six Siege"))
                    .collect::<Vec<GameStatus>>()
            })
    }
}

impl Client {
    async fn get(&self, url: Url) -> Result<reqwest::Response> {
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
    async fn refresh_auth(&self) -> Result<()> {
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
    use tracing_test::traced_test;

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

    #[traced_test]
    #[tokio::test]
    async fn operators_statistics() {
        let stats = get_client()
            .await
            .get_operators(mock_player_id())
            .await
            .unwrap();
        println!("{:?}", stats);
    }

    #[traced_test]
    #[tokio::test]
    async fn maps_statistics() {
        let stats = get_client().await.get_maps(mock_player_id()).await.unwrap();
        println!("{:?}", stats);
    }

    #[tokio::test]
    async fn full_player_profiles() {
        let stats = get_client()
            .await
            .get_full_profiles(mock_player_id())
            .await
            .unwrap();
        println!("{:#?}", stats);
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

    #[tokio::test]
    async fn retreive_server_status() {
        let status = get_client().await.siege_status().await.unwrap();
        assert_eq!(status.len(), 7);
        status
            .iter()
            .for_each(|status| assert!(status.name().contains("Rainbow")));
    }
}
