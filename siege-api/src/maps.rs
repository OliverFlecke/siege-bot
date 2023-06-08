use serde::{Deserialize, Serialize};
use strum::{Display, EnumIter, EnumString};

#[derive(
    Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq, EnumString, EnumIter, Display,
)]
#[serde(rename_all = "UPPERCASE")]
pub enum Map {
    #[serde(rename = "NIGHTHAVEN LABS")]
    NighthavenLabs,
    #[serde(rename = "STADIUM BRAVO")]
    StadiumBravo,
    #[serde(rename = "CLOSE QUARTER")]
    CloseQuarter,
    #[serde(rename = "EMERALD PLAINS")]
    EmeraldPlains,
    #[serde(rename = "BANK V2")]
    Bank,
    #[serde(rename = "BORDER V2")]
    Border,
    #[serde(rename = "CHALET V2")]
    Chalet,
    #[serde(rename = "CLUB HOUSE")]
    ClubHouse,
    Coastline,
    Consulate,
    #[serde(rename = "CONSULATE V2")]
    ConsulateV2,
    #[serde(rename = "FAVELA V2")]
    Favela,
    Fortress,
    #[serde(rename = "HEREFORD BASE")]
    HerefordBase,

    #[serde(rename = "HOUSE V3")]
    House,
    #[serde(rename = "KAFE DOSTOYEVSKY")]
    KafeDostoyevsky,
    Kanal,
    Oregon,
    #[serde(rename = "OUTBACK V2")]
    Outback,
    #[serde(rename = "PRESIDENTIAL PLANE")]
    PresidentialPlane,
    #[serde(rename = "SKYSCRAPER V2")]
    Skyscraper,
    #[serde(rename = "THEME PARK V2")]
    ThemePark,
    Tower,
    Villa,
    Yacht,
}

impl Map {
    pub fn image(&self) -> &str {
        match self {
            Self::NighthavenLabs => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/57i2PyuzpgVFzOvLUSAItO/636e57b198377a5a5d1d35492b52b808/Nighthaven_labs_screen.jpg",
            Self::StadiumBravo => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/4sQkH6y0h79oYEHuWHGAv/0103ee95bd83c8e222b32f7784e323da/r6s_maps_stadium.jpg",
            Self::CloseQuarter => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/6u3cuepKWT8IFdaGznfc3k/d470334bddf5d6313c15879cde524615/r6s_maps_closequarters.jpg",
            Self::EmeraldPlains => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/1IGW5GG24TGEv3q8bRc9aJ/a73e0dc1fd385b4afd32cd3a2592a294/r6s_maps_emeraldplains__1_.jpg",
            Self::Bank => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/6ilgtuzucX7hEu2MvjhRtp/0bb6e106d78625ea218a572fbb7a5157/r6-maps-bank.jpg",
            Self::Border => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/4hqsrL3cokFqedkfjiEaGf/c73f6714b535263a18e4de2ca2405dd1/r6-maps-border__1_.jpg",
            Self::Chalet => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/Km3ZJUM7ZMVbGsi6gad5Y/c48162371342d9f15386c77a3766315b/r6-maps-chalet.jpg",
            Self::ClubHouse => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/Km3ZJUM7ZMVbGsi6gad5Y/c48162371342d9f15386c77a3766315b/r6-maps-chalet.jpg",
            Self::Coastline => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/5GfAQ3pXCJnDqiqaDH3Zic/db1722cd699bb864ee8f7b0db951b0c3/r6-maps-coastline.jpg",
            Self::Consulate | Self::ConsulateV2 => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/6PR2sBla9E6TNurVUfJ0mc/860cab16eb1d4cd27ea356a1c3fe9591/r6-maps-consulate.jpg",
            Self::Favela => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/6PR2sBla9E6TNurVUfJ0mc/860cab16eb1d4cd27ea356a1c3fe9591/r6-maps-consulate.jpg",
            Self::Fortress => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/1MrLwvq61aSSvvUj3dDiZg/18e267c79b8015a1af509a2e5694b18b/r6-maps-fortress.jpg",
            Self::HerefordBase => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/1QHhMYSliWgWXFLxZj19hz/44197c1d98498d8a77618076a19ce538/r6-maps-hereford.jpg",
            Self::House => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/28OaEZAY3stNFr0wSvW9MB/c7acc97d43486349763acab3c1564414/r6-maps-house.jpg",
            Self::KafeDostoyevsky => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/2nIuPSHvbM57TK90VSwBEm/70144ada56cf1ba72103aeb4ece9ed1a/r6-maps-kafe.jpg",
            Self::Kanal => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/4VHR8uZRGkHqvtZxtmibtc/da988c2cab37f1cb186535fc9ba40bea/r6-maps-kanal.jpg",
            Self::Oregon => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/Z9a0gU7iR0vfcbXtoJUOW/42ad6aabbd189fbcd74c497627f1624e/r6-maps-oregon.jpg",
            Self::Outback => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/1vqGVW6pqBZlLKp4h86NnB/08a7e337c0cfa604cde79e755fedb397/r6-maps-outback.jpg",
            Self::PresidentialPlane => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/3YSN2V0HWsddcQq82Iqihn/d3e03012e8909be26f8274b7f9b3bf19/r6-maps-plane.jpg",
            Self::Skyscraper => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/7vblsbhmSPLsI3pQJ5Dqx9/f213af09981f5c8ec9b71fb0c3f9dcdd/r6-maps-skyscraper.jpg",
            Self::ThemePark => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/2immPCOZj6tTHMM9zeBg5B/cf09c9c75bc2e70dd38ebf0a12bdb9a2/r6-maps-themepark.jpg",
            Self::Tower => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/6ZMBunxANmzTNr42wwzggb/3a19c506f9e3f910e34da21095686fa9/r6-maps-tower.jpg",
            Self::Villa => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/Io6dxNeHbCbJoF9WLJf9s/ebf89b009affba37df84dcf1934c74e0/r6-maps-villa.jpg",
            Self::Yacht => "https://staticctf.ubisoft.com/J3yJr34U2pZ2Ieem48Dwy9uqj5PNUQTn/smDP6lSSaB6Daa7bLZxHZ/d6cc60d76e553e91503a474ff0bc148b/r6-maps-yacht.jpg",
        }
    }
}

#[cfg(test)]
mod test {
    use reqwest::Url;
    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn parse() {
        assert_eq!("Favela".parse::<Map>().unwrap(), Map::Favela);
        assert_eq!(
            "NighthavenLabs".parse::<Map>().unwrap(),
            Map::NighthavenLabs
        );
    }

    #[test]
    fn display() {
        assert_eq!(Map::Favela.to_string(), "Favela");
        assert_eq!(Map::NighthavenLabs.to_string(), "NighthavenLabs");
    }

    #[test]
    fn image_is_valid() {
        Map::iter().for_each(|x| {
            assert!(Url::parse(x.image()).is_ok());
        });
    }
}
