use uuid::Uuid;

use crate::auth::{Auth, ConnectError};
use crate::constants::{UBI_APP_ID, UBI_USER_AGENT};
use crate::models::{PlaytimeProfile, PlaytimeResponse};

struct Client;

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
}

#[cfg(test)]
mod test {
    use chrono::{DateTime, Duration};

    use super::*;

    fn mock_player_id() -> Uuid {
        Uuid::parse_str("e7679633-31ff-4f44-8cfd-d0ff81e2c10a").expect("this is a valid guid")
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
