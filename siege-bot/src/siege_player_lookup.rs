use std::{
    collections::HashMap,
    fs::{read_to_string, write},
    sync::Arc,
};

use serenity::{
    model::prelude::UserId,
    prelude::{RwLock, TypeMapKey},
};
use uuid::Uuid;

pub struct SiegePlayerLookup;

impl TypeMapKey for SiegePlayerLookup {
    type Value = Arc<RwLock<PlayerLookup>>;
}

static PATH: &str = ".players.json";

#[derive(Debug, Default)]
pub struct PlayerLookup(HashMap<UserId, Uuid>);

impl PlayerLookup {
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let map = match read_to_string(PATH) {
            Ok(content) => serde_json::from_str(content.as_str())?,
            Err(err) => {
                tracing::warn!("Failed to read players. Creating default. Error: {err:?}");
                HashMap::default()
            }
        };

        Ok(Self(map))
    }

    /// Get the Ubisoft ID for a Siege player from their Discord ID.
    pub fn get(&self, id: &UserId) -> Option<&Uuid> {
        self.0.get(id)
    }

    /// Insert a Discord user's Ubisoft ID for later lookup.
    pub fn insert(&mut self, id: &UserId, siege_id: Uuid) -> Result<(), std::io::Error> {
        self.0.insert(*id, siege_id);
        self.persist()
    }

    fn persist(&self) -> Result<(), std::io::Error> {
        let content =
            serde_json::to_string_pretty(&self.0).expect("should always be serializeable");
        write(PATH, content)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn load_from_disk() {
        assert!(PlayerLookup::load().is_ok());
    }

    #[test]
    fn insert_and_get_player() {
        let siege_id =
            Uuid::parse_str("68830784-0ff1-43c7-bbac-90c1e537d1cc").expect("this is a valid guid");
        let discord_id = UserId::from(1290213);
        let mut lookup = PlayerLookup::default();

        // Act - add
        lookup
            .insert(&discord_id, siege_id)
            .expect("should be able to persist");

        // Act
        let retrieved_siege_id = *lookup.get(&discord_id).unwrap();

        assert_eq!(siege_id, retrieved_siege_id);
    }
}
