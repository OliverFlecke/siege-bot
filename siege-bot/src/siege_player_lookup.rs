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

#[derive(Debug)]
pub struct PlayerLookup {
    filename: String,
    users: HashMap<UserId, Uuid>,
}

impl PlayerLookup {
    pub fn load(filename: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let users = match read_to_string(filename)
            .or_else(|_| read_to_string(format!("/config/{filename}")))
        {
            Ok(content) => serde_json::from_str(content.as_str())?,
            Err(err) => {
                tracing::warn!("Failed to read players. Creating default. Error: {err:?}");
                HashMap::default()
            }
        };

        Ok(Self {
            filename: filename.to_owned(),
            users,
        })
    }

    /// Get the Ubisoft ID for a Siege player from their Discord ID.
    pub fn get(&self, id: &UserId) -> Option<&Uuid> {
        self.users.get(id)
    }

    /// Insert a Discord user's Ubisoft ID for later lookup.
    pub fn insert(&mut self, id: &UserId, siege_id: Uuid) -> Result<(), std::io::Error> {
        self.users.insert(*id, siege_id);
        self.persist()
    }

    fn persist(&self) -> Result<(), std::io::Error> {
        let content =
            serde_json::to_string_pretty(&self.users).expect("should always be serializeable");
        write(self.filename.as_str(), content)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use tempfile::NamedTempFile;

    #[test]
    fn load_from_disk_without_existing_file() {
        assert!(PlayerLookup::load("not existing file").is_ok());
    }

    #[test]
    fn load_exsiting_file() {
        let siege_id =
            Uuid::parse_str("68830784-0ff1-43c7-bbac-90c1e537d1cc").expect("this is a valid guid");
        let discord_id = UserId::from(1290213);

        // Setup lookup and write to desk.
        let mut file = NamedTempFile::new().unwrap();
        let filename = file.path().to_str().unwrap().to_string();

        // Write an empty JSON object to the file.
        // This is just to have some valid content that serde can parse.
        use std::io::Write;
        writeln!(file, "{{}}").unwrap();
        let mut lookup = PlayerLookup::load(filename.as_str()).unwrap();

        lookup
            .insert(&discord_id, siege_id)
            .expect("should be able to persist");

        // Act - Load from the exsiting file
        let lookup = PlayerLookup::load(filename.as_str()).unwrap();

        // Assert
        assert_eq!(*lookup.get(&discord_id).unwrap(), siege_id);

        drop(file);
    }

    #[test]
    fn insert_and_get_player() {
        let siege_id =
            Uuid::parse_str("68830784-0ff1-43c7-bbac-90c1e537d1cc").expect("this is a valid guid");
        let discord_id = UserId::from(1290213);
        let mut lookup = PlayerLookup::load("temp").unwrap();

        // Act - add
        lookup
            .insert(&discord_id, siege_id)
            .expect("should be able to persist");

        // Act
        let retrieved_siege_id = *lookup.get(&discord_id).unwrap();

        assert_eq!(siege_id, retrieved_siege_id);
    }

    #[test]
    fn debug() {
        let lookup = PlayerLookup {
            filename: "some name".to_string(),
            users: HashMap::default(),
        };

        assert_eq!(
            format!("{lookup:?}"),
            "PlayerLookup { filename: \"some name\", users: {} }"
        );
    }
}
