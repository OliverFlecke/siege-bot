mod data;

use derive_getters::Getters;
use serde::Deserialize;
use strum::{Display, EnumIter, EnumString};

use crate::game_models::{Health, Role, Season, Side, Speed};

pub use data::get_operator_details;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, EnumString, Display, EnumIter)]
pub enum Operator {
    Brava,
    Solis,
    Grim,
    Sens,
    Azami,
    Thorn,
    Osa,
    Thunderbird,
    Flores,
    Aruni,
    Zero,
    Ace,
    Melusi,
    Oryx,
    Iana,
    Wamai,
    Kali,
    Amaru,
    Goyo,
    #[serde(rename = "Nokk")]
    Nøkk,
    Warden,
    Mozzie,
    Gridlock,
    Nomad,
    Kaid,
    Clash,
    Maverick,
    Maestro,
    Alibi,
    Lion,
    Finka,
    Vigil,
    Dokkaebi,
    Zofia,
    Ela,
    Ying,
    Lesion,
    Mira,
    Jackal,
    Hibana,
    Echo,
    Caveira,
    #[serde(rename = "Capitao")]
    Capitão,
    Blackbeard,
    Valkyrie,
    Buck,
    Frost,
    Mute,
    Sledge,
    Smoke,
    Thatcher,
    Ash,
    Castle,
    Pulse,
    Thermite,
    Montagne,
    Twitch,
    Doc,
    Rook,
    #[serde(rename = "Jager")]
    Jäger,
    Bandit,
    Blitz,
    Iq,
    Fuze,
    Glaz,
    Tachanka,
    Kapkan,
    Recruit,
}

impl Operator {
    /// Get a URL for the avatar for this operator.
    pub fn avatar_url(&self) -> String {
        format!(
            "https://r6operators.marcopixel.eu/icons/png/{}.png",
            self.to_string().to_lowercase()
        )
    }
}

#[derive(Debug, Getters)]
pub struct OperatorDetails {
    realname: String,
    birthplace: String,
    age: u8,
    date_of_birth: String, // TODO: chrono::NaiveDate,
    season_introduced: Season,
    health: Health,
    speed: Speed,
    unit: String,
    country_code: String,
    roles: Vec<Role>,
    side: Side,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse() {
        assert_eq!("Nøkk".parse::<Operator>().unwrap(), Operator::Nøkk);
        assert_eq!("Jäger".parse::<Operator>().unwrap(), Operator::Jäger);
        assert_eq!("Aruni".parse::<Operator>().unwrap(), Operator::Aruni);
    }
}
