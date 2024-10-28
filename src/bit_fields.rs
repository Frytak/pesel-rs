use super::*;

pub const CONTROL_SECTION_SIZE: u8 = 4;
pub const ORDINAL_SECTION_SIZE: u8 = 14;
pub const DAY_SECTION_SIZE: u8 = 7;
pub const MONTH_SECTION_SIZE: u8 = 5;
pub const YEAR_SECTION_SIZE: u8 = 7;

pub const CONTROL_SECTION_SHIFT: u8 = 0;
pub const ORDINAL_SECTION_SHIFT: u8 = CONTROL_SECTION_SIZE + 5;
pub const DAY_SECTION_SHIFT: u8 = ORDINAL_SECTION_SHIFT + ORDINAL_SECTION_SIZE + 5;
pub const MONTH_SECTION_SHIFT: u8 = DAY_SECTION_SHIFT + DAY_SECTION_SIZE + 5;
pub const YEAR_SECTION_SHIFT: u8 = MONTH_SECTION_SHIFT + MONTH_SECTION_SIZE + 5;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pesel(u64);

impl From<Pesel> for u64 {
    fn from(value: Pesel) -> Self {
        (&value).into()
    }
}

impl From<&Pesel> for u64 {
    fn from(value: &Pesel) -> Self {
        value.control_section() as u64
        + (value.ordinal_section() as u64) * 10u64.pow(1)
        + (value.day_section() as u64) * 10u64.pow(5)
        + (value.month_section() as u64) * 10u64.pow(7)
        + (value.year_section() as u64) * 10u64.pow(9)
    }
}

impl TryFrom<u64> for Pesel {
    type Error = ValidationError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        validate(value)?;
        let pesel = Pesel(
            ((crate::day_section(value) as u64) << DAY_SECTION_SHIFT as u64)
            + ((crate::month_section(value) as u64) << MONTH_SECTION_SHIFT as u64)
            + ((crate::year_section(value) as u64) << YEAR_SECTION_SHIFT as u64)
            + ((crate::ordinal_section(value) as u64) << ORDINAL_SECTION_SHIFT as u64)
            + ((crate::control_section(value) as u64) << CONTROL_SECTION_SHIFT as u64)
        );
        Ok(pesel)
    }
}

impl From<crate::human_redable::Pesel> for Pesel {
    fn from(value: crate::human_redable::Pesel) -> Self {
        Self(
            ((value.day_section() as u64) << DAY_SECTION_SHIFT as u64)
            + ((value.month_section() as u64) << MONTH_SECTION_SHIFT as u64)
            + ((value.year_section() as u64) << YEAR_SECTION_SHIFT as u64)
            + ((value.ordinal_section() as u64) << ORDINAL_SECTION_SHIFT as u64)
            + ((value.control_section() as u64) << CONTROL_SECTION_SHIFT as u64)
        )
    }
}

impl PeselTrait for Pesel {
    fn day_section(&self) -> u8 {
        (self.0 >> DAY_SECTION_SHIFT & (2u64.pow(DAY_SECTION_SIZE as u32) - 1)) as u8
    }

    fn month_section(&self) -> u8 {
        (self.0 >> MONTH_SECTION_SHIFT & (2u64.pow(MONTH_SECTION_SIZE as u32) - 1)) as u8
    }

    fn year_section(&self) -> u8 {
        (self.0 >> YEAR_SECTION_SHIFT & (2u64.pow(YEAR_SECTION_SIZE as u32) - 1)) as u8
    }

    fn ordinal_section(&self) -> u16 {
        (self.0 >> ORDINAL_SECTION_SHIFT & (2u64.pow(ORDINAL_SECTION_SIZE as u32) - 1)) as u16
    }

    fn control_section(&self) -> u8 {
        (self.0 >> CONTROL_SECTION_SHIFT & (2u64.pow(CONTROL_SECTION_SIZE as u32) - 1)) as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::LazyLock;

    static PESEL1: LazyLock<Pesel> = LazyLock::new(|| Pesel::try_from(02290486168).unwrap());
    static PESEL2: LazyLock<Pesel> = LazyLock::new(|| Pesel::try_from(01302534699).unwrap());
    static PESEL3: LazyLock<Pesel> = LazyLock::new(|| Pesel::try_from(00010128545).unwrap());
    static PESEL4: LazyLock<Pesel> = LazyLock::new(|| Pesel::try_from(98250993285).unwrap());
    static PESEL5: LazyLock<Pesel> = LazyLock::new(|| Pesel::try_from(60032417874).unwrap());

    #[test]
    fn day_section() {
        assert_eq!(PESEL1.day_section(), 04);
        assert_eq!(PESEL2.day_section(), 25);
        assert_eq!(PESEL3.day_section(), 01);
        assert_eq!(PESEL4.day_section(), 09);
        assert_eq!(PESEL5.day_section(), 24);
    }

    #[test]
    fn month_section() {
        assert_eq!(PESEL1.month_section(), 29);
        assert_eq!(PESEL2.month_section(), 30);
        assert_eq!(PESEL3.month_section(), 01);
        assert_eq!(PESEL4.month_section(), 25);
        assert_eq!(PESEL5.month_section(), 03);
    }

    #[test]
    fn year_section() {
        assert_eq!(PESEL1.year_section(), 02);
        assert_eq!(PESEL2.year_section(), 01);
        assert_eq!(PESEL3.year_section(), 00);
        assert_eq!(PESEL4.year_section(), 98);
        assert_eq!(PESEL5.year_section(), 60);
    }

    #[test]
    fn ordinal_section() {
        assert_eq!(PESEL1.ordinal_section(), 8616);
        assert_eq!(PESEL2.ordinal_section(), 3469);
        assert_eq!(PESEL3.ordinal_section(), 2854);
        assert_eq!(PESEL4.ordinal_section(), 9328);
        assert_eq!(PESEL5.ordinal_section(), 1787);
    }

    #[test]
    fn control_section() {
        assert_eq!(PESEL1.control_section(), 8);
        assert_eq!(PESEL2.control_section(), 9);
        assert_eq!(PESEL3.control_section(), 5);
        assert_eq!(PESEL4.control_section(), 5);
        assert_eq!(PESEL5.control_section(), 4);
    }

    #[test]
    fn day() {
        assert_eq!(PESEL1.day(), 04);
        assert_eq!(PESEL2.day(), 25);
        assert_eq!(PESEL3.day(), 01);
        assert_eq!(PESEL4.day(), 09);
        assert_eq!(PESEL5.day(), 24);
    }

    #[test]
    fn month() {
        assert_eq!(PESEL1.month(), 09);
        assert_eq!(PESEL2.month(), 10);
        assert_eq!(PESEL3.month(), 01);
        assert_eq!(PESEL4.month(), 05);
        assert_eq!(PESEL5.month(), 03);
    }

    #[test]
    fn year() {
        assert_eq!(PESEL1.year(), 2002);
        assert_eq!(PESEL2.year(), 2001);
        assert_eq!(PESEL3.year(), 1900);
        assert_eq!(PESEL4.year(), 2098);
        assert_eq!(PESEL5.year(), 1960);
    }

    #[test]
    fn date_of_birth() {
        assert_eq!(PESEL1.date_of_birth(), NaiveDate::from_ymd_opt(2002, 09, 04).unwrap());
        assert_eq!(PESEL2.date_of_birth(), NaiveDate::from_ymd_opt(2001, 10, 25).unwrap());
        assert_eq!(PESEL3.date_of_birth(), NaiveDate::from_ymd_opt(1900, 01, 01).unwrap());
        assert_eq!(PESEL4.date_of_birth(), NaiveDate::from_ymd_opt(2098, 05, 09).unwrap());
        assert_eq!(PESEL5.date_of_birth(), NaiveDate::from_ymd_opt(1960, 03, 24).unwrap());
    }

    #[test]
    fn gender() {
        assert_eq!(PESEL1.gender(), Gender::Female);
        assert_eq!(PESEL2.gender(), Gender::Male);
        assert_eq!(PESEL3.gender(), Gender::Female);
        assert_eq!(PESEL4.gender(), Gender::Female);
        assert_eq!(PESEL5.gender(), Gender::Male);
    }

    #[test]
    fn invalid_pesels() {
        assert_eq!(Pesel::try_from(4355), Err(ValidationError::TooShort(4)));
        assert_eq!(Pesel::try_from(435585930294485), Err(ValidationError::TooLong(15)));
        assert_eq!(Pesel::try_from(99990486167), Err(ValidationError::BirthDate));
        assert_eq!(Pesel::try_from(02290486167), Err(ValidationError::ControlDigit));
    }
}

