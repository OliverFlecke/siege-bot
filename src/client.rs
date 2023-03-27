use reqwest::Url;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::{Auth, ConnectError};
use crate::constants::{UBI_APP_ID, UBI_USER_AGENT};
use crate::models::{FullProfile, PlatformFamily, PlayType, PlaytimeProfile, PlaytimeResponse};

pub struct Client;

impl Client {
    pub async fn get_playtime(&self, player_id: Uuid) -> Result<PlaytimeProfile, ConnectError> {
        // TODO: Can this be made more readable?
        // TODO: Should `spaceId` be a parameter?
        let url = format!("https://public-ubiservices.ubi.com/v1/profiles/stats?profileIds={player_id}&spaceId=0d2ae42d-4c27-4cb7-af6c-2099062302bb&statNames=PPvPTimePlayed,PPvETimePlayed,PTotalTimePlayed,PClearanceLevel");

        let connected = Auth::from_environment().connect().await?;

        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("Authorization", format!("Ubi_v1 t={}", connected.ticket()))
            .header("User-Agent", UBI_USER_AGENT)
            .header("Ubi-AppId", UBI_APP_ID)
            .header("Ubi-LocalCode", "en-US")
            .header("Ubi-SessionId", connected.session_id().to_string())
            .header("expiration", connected.expiration().to_string())
            .send()
            .await
            .map_err(|_| ConnectError::ConnectionError)?;

        // println!(
        //     "{}",
        //     response
        //         .text()
        //         .await
        //         .map_err(|_| ConnectError::UnexpectedResponse)?
        // );

        let parsed = response
            .json::<PlaytimeResponse>()
            .await
            .map_err(|_| ConnectError::UnexpectedResponse)?;

        Ok(parsed.profiles()[0])
    }

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

        let connected = Auth::from_environment().connect().await?;
        let client = reqwest::Client::new();
        let response = client
            .get(url)
            .header("Authorization", format!("Ubi_v1 t={}", connected.ticket()))
            .header("User-Agent", UBI_USER_AGENT)
            .header("Ubi-AppId", UBI_APP_ID)
            .header("Ubi-LocalCode", "en-US")
            .header("Ubi-SessionId", connected.session_id().to_string())
            .header("expiration", connected.expiration().to_string())
            .send()
            .await
            .map_err(|_| ConnectError::ConnectionError)?;

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
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Duration};

    use super::*;

    fn mock_player_id() -> Uuid {
        Uuid::parse_str("e7679633-31ff-4f44-8cfd-d0ff81e2c10a").expect("this is a valid guid")
    }

    #[tokio::test]
    async fn full_player_profiles() {
        let client = Client {};

        let profile = client.get_full_profiles(mock_player_id()).await.unwrap();
    }

    #[tokio::test]
    async fn playtime() {
        let player_id = mock_player_id();
        let client = Client {};

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
