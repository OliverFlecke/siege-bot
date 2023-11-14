use derive_getters::Getters;
use serde::Deserialize;
use strum::Display;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Getters)]
#[serde(rename_all = "PascalCase")]
pub struct GameStatus {
    #[serde(rename = "AppID ", deserialize_with = "super::mappers::string_to_uuid")]
    app_id: Option<Uuid>,
    // m_d_m: usize, // Ignored for now, as this is unused
    #[serde(
        rename = "SpaceID",
        deserialize_with = "super::mappers::string_to_uuid"
    )]
    space_id: Option<Uuid>,
    category: String,
    name: String,
    platform: Platform,
    status: Status,
    maintenance: Option<bool>,
    impacted_features: Vec<String>,
}

/// Represents the different statuses. This is not an exhausted list, as
/// there is no documentation from Ubisoft on the actual options.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Display)]
pub enum Status {
    Online,
    Degraded,
    Interrupted,
}

#[derive(Debug, Clone, Copy, Deserialize, Display)]
pub enum Platform {
    Android,
    #[serde(rename = "GEFORCE NOW")]
    GeforeNow,
    #[serde(rename = "I-PAD")]
    IPAD,
    Luna,
    PC,
    PCWeb,
    PS3,
    PS4,
    PS5,
    SWITCH,
    WII,
    WIIU,
    X360,
    #[serde(rename = "XBOX SERIES X")]
    XboxSeriesX,
    #[serde(rename = "XBOXONE")]
    XboxOne,
    #[serde(rename = "iOS")]
    IOS,
    #[serde(other)]
    Unkwon,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_good_status() {
        let content = std::fs::read_to_string("../samples/game-status.json").unwrap();
        let _: Vec<GameStatus> = serde_json::from_str(content.as_str()).unwrap();
    }

    #[test]
    fn parse_bad_status() {
        let content = std::fs::read_to_string("../samples/game-status-with-error.json").unwrap();
        let _: Vec<GameStatus> = serde_json::from_str(content.as_str()).unwrap();
    }
}
