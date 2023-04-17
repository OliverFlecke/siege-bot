use derive_getters::Getters;
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Getters)]
#[serde(rename_all = "PascalCase")]
pub struct GameStatus {
    #[serde(rename = "AppID")]
    app_id: Option<Uuid>,
    // m_d_m: usize, // Ignored for now, as this is unused
    #[serde(rename = "SpaceID")]
    space_id: String,
    category: String,
    name: String,
    platform: String, // Could be enum
    status: String,   // Could be enum
    maintenance: Option<String>,
    impacted_features: Vec<String>,
}
