use strum::{Display, EnumIter, FromRepr};

#[repr(u8)]
#[derive(Debug, Clone, Copy, FromRepr, Display, EnumIter, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
    Unranked,

    CopperV,
    CopperIV,
    CopperIII,
    CopperII,
    CopperI,

    BronzeV,
    BronzeIV,
    BronzeIII,
    BronzeII,
    BronzeI,

    SilverV,
    SilverIV,
    SilverIII,
    SilverII,
    SilverI,

    GoldV,
    GoldIV,
    GoldIII,
    GoldII,
    GoldI,

    PlatinumV,
    PlatinumIV,
    PlatinumIII,
    PlatinumII,
    PlatinumI,

    EmeraldV,
    EmeraldIV,
    EmeraldIII,
    EmeraldII,
    EmeraldI,

    DiamondV,
    DiamondIV,
    DiamondIII,
    DiamondII,
    DiamondI,

    Champions,
}

impl Rank {
    pub fn from_mmr(mmr: u64) -> Rank {
        use Rank::*;
        match mmr {
            0..=999 => Unranked,
            1000..=1099 => CopperV,
            1100..=1199 => CopperIV,
            1200..=1299 => CopperIII,
            1300..=1399 => CopperII,
            1400..=1499 => CopperI,
            1500..=1599 => BronzeV,
            1600..=1699 => BronzeIV,
            1700..=1799 => BronzeIII,
            1800..=1899 => BronzeII,
            1900..=1999 => BronzeI,
            2000..=2099 => SilverV,
            2100..=2199 => SilverIV,
            2200..=2299 => SilverIII,
            2300..=2399 => SilverII,
            2400..=2499 => SilverI,
            2500..=2599 => GoldV,
            2600..=2699 => GoldIV,
            2700..=2799 => GoldIII,
            2800..=2899 => GoldII,
            2900..=2999 => GoldI,
            3000..=3099 => PlatinumV,
            3100..=3199 => PlatinumIV,
            3200..=3299 => PlatinumIII,
            3300..=3399 => PlatinumII,
            3400..=3499 => PlatinumI,
            3500..=3599 => EmeraldV,
            3600..=3699 => EmeraldIV,
            3700..=3799 => EmeraldIII,
            3800..=3899 => EmeraldII,
            3900..=3999 => EmeraldI,
            4000..=4099 => DiamondV,
            4100..=4199 => DiamondIV,
            4200..=4299 => DiamondIII,
            4300..=4399 => DiamondII,
            4400..=4499 => DiamondI,
            _ => Champions,
        }
    }
}

#[cfg(test)]
mod test {
    use std::iter::zip;

    use strum::IntoEnumIterator;

    use super::*;

    #[test]
    fn from_repr() {
        for rank in Rank::iter() {
            assert_eq!(rank, Rank::from_repr(rank as u8).unwrap());
        }
    }

    #[test]
    fn from_mmr() {
        for (rank, mmr) in zip(Rank::iter(), (900..=4500).step_by(100)) {
            assert_eq!(rank, Rank::from_mmr(mmr));
        }
    }
}
