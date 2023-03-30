use serde::Deserialize;
use strum::{Display, EnumString};

#[derive(Debug, Deserialize, EnumString, Display)]
pub enum Season {
    Y8S1,
    Y7S4,
    Y7S3,
    Y7S2,
    Y7S1,
    Y6S4,
    Y6S3,
    Y6S2,
    Y6S1,
    Y5S4,
    Y5S3,
    Y5S2,
    Y5S1,
    Y4S4,
    Y4S3,
    Y4S2,
    Y4S1,
    Y3S4,
    Y3S3,
    Y3S2,
    Y3S1,
    Y2S4,
    Y2S3,
    Y2S2,
    Y2S1,
    Y1S4,
    Y1S3,
    Y1S2,
    Y1S1,
    Y0S0,
}

#[derive(Debug, Deserialize)]
pub enum Health {
    Low = 1,
    Medium = 2,
    High = 3,
}

#[derive(Debug, Deserialize)]
pub enum Speed {
    Slow = 1,
    Normal = 2,
    Fast = 3,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Side {
    Attacker,
    Defender,
}

#[derive(Debug, Deserialize)]
pub enum Role {
    Anchor,
    AntiHardBreach,
    AntiRoam,
    AreaDenial,
    BackLine,
    Buff,
    CoveringFire,
    CrowdControl,
    Disable,
    Flank,
    FrontLine,
    HardBreach,
    IntelDenier,
    IntelGatherer,
    Roam,
    Secure,
    Shield,
    SoftBreach,
    Trap,
    AntiGadget,
}
