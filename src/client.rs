use chrono::{Months, NaiveDate, Utc};
use reqwest::{RequestBuilder, Url};
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{ConnectError, ConnectResponse};
use crate::constants::{UBI_APP_ID, UBI_SERVICES_URL, UBI_USER_AGENT};
use crate::models::{
    FullProfile, OperatorStatisticResponse, PlatformFamily, PlatformType, PlayType,
    PlaytimeProfile, PlaytimeResponse,
};

#[derive(Debug)]
pub struct Client {
    auth: ConnectResponse,
    client: reqwest::Client,
}

impl From<ConnectResponse> for Client {
    fn from(auth: ConnectResponse) -> Self {
        Self {
            auth,
            client: reqwest::Client::new(),
        }
    }
}

impl Client {
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

        let response = self
            .client
            .get(url)
            .set_headers(&self.auth)
            .send()
            .await
            .map_err(|_| ConnectError::ConnectionError)?;

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

        let response = self
            .client
            .get(url)
            .set_headers(&self.auth)
            .send()
            .await
            .map_err(|_| ConnectError::ConnectionError)?;

        // Helper structs to extract the unnecssary nesting from the API.
        #[derive(Deserialize)]
        struct Response {
            platform_families_full_profiles: Vec<PlatformFamiliesFullProfile>,
        }
        #[derive(Deserialize)]
        struct PlatformFamiliesFullProfile {
            #[allow(dead_code)]
            platform_family: PlatformFamily,
            board_ids_full_profiles: Vec<Board>,
        }
        #[derive(Deserialize)]
        struct Board {
            #[allow(dead_code)]
            board_id: PlayType,
            full_profiles: Vec<FullProfile>,
        }

        let profile = response.json::<Response>().await.map_err(|err| {
            println!("Err: {err:?}");
            ConnectError::UnexpectedResponse
        })?;

        Ok(profile.platform_families_full_profiles[0]
            .board_ids_full_profiles
            .iter()
            .map(|x| x.full_profiles[0])
            .collect::<Vec<_>>())
    }

    pub async fn get_statistics(&self, player_id: Uuid) -> Result<(), Box<dyn std::error::Error>> {
        let platform = PlatformType::Uplay;
        let statistics = vec!["operatorpvp_kills"];

        let url = format!(
            "{}/v1/spaces/{space}/sandboxes/{sandbox}/",
            UBI_SERVICES_URL,
            space = platform.get_space(),
            sandbox = platform.get_sandbox(),
        );
        let url = format!("{url}/playerstats2/statistics");
        let url = Url::parse_with_params(
            url.as_str(),
            &[
                ("populations", player_id.to_string()),
                ("statistics", statistics.join(",")),
            ],
        )
        .expect("is a valid url");

        let response = self.client.get(url).set_headers(&self.auth).send().await?;

        println!("{}", response.text().await?);
        todo!()
    }

    pub async fn get_operators(
        &self,
        player_id: Uuid,
    ) -> Result<OperatorStatisticResponse, Box<dyn std::error::Error>> {
        let url = create_operators_url(player_id);
        println!("URL: {url}");

        let response = self.client.get(url).set_headers(&self.auth).send().await?;

        println!("Status: {}", response.status());
        // println!("Body: {}", response.text().await?);

        Ok(response.json::<OperatorStatisticResponse>().await?)
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

fn create_operators_url(player_id: Uuid) -> Url {
    // Url::parse("https://prod.datadev.ubisoft.com/v1/profiles/e7679633-31ff-4f44-8cfd-d0ff81e2c10a/playerstats?spaceId=5172a557-50b5-4665-b7db-e3f2e8c5041d&view=current&aggregation=operators&gameMode=all,ranked,casual,unranked&platformGroup=PC&teamRole=all,Attacker,Defender").unwrap()

    fn format_date(date: NaiveDate) -> String {
        date.format("%Y%m%d").to_string()
    }

    // NOTE: seems like these cannot be more than three (somthing is off with my math) months apart.
    // Or maybe it just cannot go earlier than 2022-11-25. Not sure what is special about that date.
    // let start_date = NaiveDate::from_ymd_opt(2022, 11, 25).expect("is a valid date"); // TODO: Find a proper default
    let end_date = Utc::now().date_naive();
    let start_date = end_date
        .checked_sub_months(Months::new(3))
        .expect("should be valid");

    let url = format!("https://prod.datadev.ubisoft.com/v1/profiles/{player_id}/playerstats");
    Url::parse_with_params(
        url.as_str(),
        &[
            ("view", "current"),
            ("platformGroup", "PC"),
            ("aggregation", "operators"), // TODO: Could be an argument
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
            ("startDate", format_date(start_date).as_str()),
            ("endDate", format_date(end_date).as_str()),
        ],
    )
    .expect("is a valid url")
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Duration};

    use crate::auth::Auth;

    use super::*;

    fn mock_player_id() -> Uuid {
        Uuid::parse_str("e7679633-31ff-4f44-8cfd-d0ff81e2c10a").expect("this is a valid guid")
    }

    async fn create_client_from_environment() -> Client {
        let auth = Auth::from_environment();

        Into::<Client>::into(auth.connect().await.unwrap())
    }

    #[test]
    fn operators_url() {
        let expected = "https://prod.datadev.ubisoft.com/v1/profiles/e7679633-31ff-4f44-8cfd-d0ff81e2c10a/playerstats?spaceId=5172a557-50b5-4665-b7db-e3f2e8c5041d&view=current&aggregation=operators&gameMode=all,ranked,casual,unranked&platformGroup=PC&teamRole=all,Attacker,Defender";

        let actual = create_operators_url(mock_player_id());
        assert_eq!(actual.as_str(), expected);
    }

    #[tokio::test]
    async fn operators_statistics() {
        let client = create_client_from_environment().await;

        let stats = client.get_operators(mock_player_id()).await.unwrap();
        println!("{:?}", stats);
    }

    #[tokio::test]
    async fn full_player_profiles() {
        let client = create_client_from_environment().await;

        _ = client.get_full_profiles(mock_player_id()).await.unwrap();
    }

    #[tokio::test]
    async fn playtime() {
        let player_id = mock_player_id();
        let client = create_client_from_environment().await;
        let playtime = client.get_playtime(player_id).await.unwrap();

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
