use serde::Deserialize;
use strum::{Display, EnumIter, EnumString};

pub use crate::data::operator::get_operator_details;

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq, EnumString, Display, EnumIter)]
pub enum Operator {
    Ram,
    Fenrir,
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
    #[serde(rename = "No Class")]
    NoClass,
    #[serde(other)]
    Unknown,
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

#[cfg(test)]
mod test {
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn parse() {
        assert_eq!("Nøkk".parse::<Operator>().unwrap(), Operator::Nøkk);
        assert_eq!("Jäger".parse::<Operator>().unwrap(), Operator::Jäger);
        assert_eq!("Aruni".parse::<Operator>().unwrap(), Operator::Aruni);
    }

    #[test]
    fn validate_avatars() {
        Operator::iter().for_each(|op| {
            assert!(reqwest::Url::parse(op.avatar_url().as_str()).is_ok());
        });
    }
}
