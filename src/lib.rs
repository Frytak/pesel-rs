//! [PESEL](https://en.wikipedia.org/wiki/PESEL) validation and detail extraction with multiple data layout implementations.
//!
//! [![Crates.io Version](https://img.shields.io/crates/v/pesel-rs?color=green)](https://crates.io/crates/pesel-rs)
//! [![Static Badge](https://img.shields.io/badge/docs-orange)](https://docs.rs/pesel-rs/latest/pesel_rs/)
//! [![Crates.io License](https://img.shields.io/crates/l/pesel-rs)](https://crates.io/crates/pesel-rs)
//!
//! # Definitions
//!
//! PESEL: `YYMMDDOOOOC`
//!
//! - `YY` - Last two digits of year of birth
//! - `MM` - Month of birth (shifted depending on year of birth as shown by the table below)
//!
//! | Year        | 1800 - 1899 | 1900 - 1999 | 2000 - 2099 | 2100 - 2199 | 2200 - 2299 |
//! |-------------|-------------|-------------|-------------|-------------|-------------|
//! | Month shift | +80         | 0           | +20         | +40         | +60         |
//!
//! - `DD` - Day of birth
//! - `OOOO` - Ordinal number, where the last digit denotes the gender ([0, 2, 4, 6, 8] = female, [1,
//!   3, 5, 7, 9] = male)
//! - `C` - Control number
//!
//! # Usage
//!
//! There are two PESEL structs provided by the crate, both implementing the [`PeselTrait`].
//!
//! - [`crate::bit_fields::Pesel`] - Stores each section of the PESEL in the following layout:
//!   `7 bits | YY | 5 bits | MM | 5 bits | DD | 5 bits | OOOO | 5 bits | C`, where in between bits
//!   are unused. Extracting each field is done using bitwise operations. You can get the human
//!   readable number using `u64::from`.
//!
//! - [`crate::human_redable::Pesel`] - Stores the PESEL as a plain number, extracting each field
//!   requires modulo and division operations, if often accessing individual fields is important to
//!   you, you should probably use [`crate::bit_fields::Pesel`].
//!
//! If you just need to validate a number or extract a specific section without using the structs,
//! you could use functions in the lib root. Most of these functions won't check if the value
//! they're returning is valid, unlike the structs who are guaranteed to always return a valid
//! value.
//!
//! # Examples
//!
//! Function that takes a name and welcomes the person based on date of birth and gender from the
//! PESEL. Implemented using [`crate::bit_fields::Pesel`] because we're mostly reading the fields.
//!
//! ```rust
//! use pesel_rs::{prelude::*, bit_fields::Pesel};
//!
//! fn welcome(first_name: &str, pesel: u64) {
//!     match Pesel::try_from(pesel) {
//!         Ok(pesel) => {
//!             if pesel.date_of_birth() > NaiveDate::from_ymd_opt(2015, 1, 1).unwrap() {
//!                 let gender = if pesel.gender() == Gender::Male { "boy" } else { "girl" };
//!                 println!("Wow {first_name}! You're such a young {gender}!");
//!             } else {
//!
//!                 println!("{first_name}, you're very old, I'm sorry 😞");
//!             }
//!         }
//!         Err(_) => println!("Huh, what you gave me doesn't seem to be a valid pesel {first_name}..."),
//!     }
//! }
//! ```
//!
//! Function finding a pesel with the oldest date of birth. Working with a generic PESEL, we
//! introduce additional bounds (required by [`PeselTrait`]).
//! ```rust
//! use pesel_rs::prelude::*;
//!
//! fn oldest<T: PeselTrait>(pesels: &[T])
//! where
//!     u64: From<T>,
//!     for<'a> u64: From<&'a T>
//! {
//!     assert!(pesels.len() > 0);
//! 
//!     let mut oldest_index = 0;
//!     pesels.iter().skip(1).enumerate().for_each(|(i, pesel)| {
//!         if pesels[oldest_index].date_of_birth() < pesel.date_of_birth() {
//!             oldest_index = i;
//!         }
//!     });
//! 
//!     let date_of_birth = pesels[oldest_index].date_of_birth();
//!     println!("PESEL nr. {oldest_index} is the oldest! Born at {date_of_birth}")
//! }
//! ```
pub mod human_redable;
pub mod bit_fields;

pub use chrono;
pub use thiserror;
#[cfg(feature = "serde")]
pub use serde;

pub mod prelude {
    pub use crate::{Gender, PeselTrait, validate};
    pub use chrono::NaiveDate;
}

use chrono::NaiveDate;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Gender {
    Male,
    Female,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    #[error("Pesel is too short.")]
    TooShort(usize),
    #[error("Pesel is too long.")]
    TooLong(usize),
    #[error("Pesel has an invalid date of birth.")]
    BirthDate,
    #[error("Pesel has an invalid control digit.")]
    ControlDigit,
}

const PESEL_WEIGHTS: [u8; 11] = [1, 3, 7, 9, 1, 3, 7, 9, 1, 3, 1];

/// # Errors
/// Returns `None` if:
/// - `month_section` is not in range of `<1,92>`
pub const fn month_from_section(month_section: u8) -> Option<u8> {
    if !(1 <= month_section && month_section <= 92) { return None; }

    Some(month_section - (((month_section / 10) / 2) * 20))
}

/// # Errors
/// Returns `None` if:
/// - `month` is not in range of `<1,12>`
/// - `year` is not in range of `<1800,2299>`
pub const fn month_to_section(month: u8, year: u16) -> Option<u8> {
    if !(1 <= month && month <= 12) { return None; }
    if !(1800 <= year && year <= 2299) { return None; }

    // TODO: Find a better conversion method
    let base = ((year / 100) - 10) as u8;
    let shift = match base {
        8 => 80,
        9 => 0,
        base => (base + 1) * 20,
    };

    Some(month + shift)
}

pub const fn year_from_sections(month_section: u8, year_section: u8) -> u16 {
    let shift = ((month_section / 10) / 2) * 2;

    (match shift {
        8 => 1800,
        shift => 1900 + (shift as u16) * 50,
    } + year_section as u16)
}

/// Trait for implementing a [PESEL](https://en.wikipedia.org/wiki/PESEL).
///
/// It's required for a PESEL to implement [`TryFrom<u64>`] and [`Into<u64>`] (for `Self` and `&Self`)
/// where the [`u64`] PESEL must be represented as a human readable number.
///
/// The only required methods are for extracting each section. The rest is computed based on that.
pub trait PeselTrait: TryFrom<u64> + Into<u64>
where
    u64: From<Self>,
    for<'a> u64: From<&'a Self> {
    /// Day of birth section.
    fn day_section(&self) -> u8;

    /// Month of birth section.
    fn month_section(&self) -> u8;

    /// Year of birth section.
    fn year_section(&self) -> u8;

    /// Ordinal section.
    fn ordinal_section(&self) -> u16;

    /// Control section.
    fn control_section(&self) -> u8;

    /// Day of birth.
    fn day(&self) -> u8 {
        self.day_section()
    }

    /// Month of birth.
    fn month(&self) -> u8 {
        match month(self) {
            Some(month) => month,
            None => unreachable!(),
        }
    }

    /// Year of birth.
    fn year(&self) -> u16 {
        year(self)
    }

    /// Date of birth.
    fn date_of_birth(&self) -> NaiveDate {
        match date_of_birth(self) {
            Some(date_of_birth) => date_of_birth,
            None => unreachable!(),
        }
    }
    
    /// Gender.
    fn gender(&self) -> Gender {
        gender(self)
    }
}

/// Extract the day of birth section.
pub fn day_section(pesel: impl Into<u64>) -> u8 {
    ((pesel.into() % 10_000_000) / 100_000) as u8
}

/// Extract the month of birth section.
pub fn month_section(pesel: impl Into<u64>) -> u8 {
    ((pesel.into() % 1_000_000_000) / 10_000_000) as u8
}

/// Extract the year of birth section.
pub fn year_section(pesel: impl Into<u64>) -> u8 {
    ((pesel.into() % 100_000_000_000) / 1_000_000_000) as u8
}

/// Extract the ordinal section.
pub fn ordinal_section(pesel: impl Into<u64>) -> u16 {
    ((pesel.into() % 100_000) / 10) as u16
}

/// Extract the control section.
pub fn control_section(pesel: impl Into<u64>) -> u8 {
    (pesel.into() % 10) as u8
}

/// Extract day of birth.
pub fn day(pesel: impl Into<u64>) -> u8 {
    day_section(pesel)
}

/// Extract month of birth.
///
/// # Errors
/// Returns `None` if:
/// - `month_section` is not in range of `<1,92>`
pub fn month(pesel: impl Into<u64>) -> Option<u8> {
    month_from_section(month_section(pesel))
}

/// Extract year of birth.
pub fn year(pesel: impl Into<u64>) -> u16 {
    let pesel = pesel.into();
    year_from_sections(month_section(pesel), year_section(pesel))
}

/// Extract date of birth.
pub fn date_of_birth(pesel: impl Into<u64>) -> Option<NaiveDate> {
    let pesel = pesel.into();
    NaiveDate::from_ymd_opt(
        year(pesel) as i32,
        match month(pesel) { Some(month) => month as u32, None => return None, },
        day(pesel) as u32
    )
}

/// Extract gender.
pub fn gender(pesel: impl Into<u64>) -> Gender {
    if ordinal_section(pesel) % 2 == 0 {
        Gender::Female
    } else {
        Gender::Male
    }
}

/// Check if the PESEL is valid.
pub fn validate(pesel: impl Into<u64>) -> Result<(), ValidationError> {
    let pesel = pesel.into();
    let mut pesel_str = pesel.to_string();

    if pesel_str.len() < 8 {
        return Err(ValidationError::TooShort(pesel_str.len()))
    }

    if pesel_str.len() > 11 {
        return Err(ValidationError::TooLong(pesel_str.len()))
    }

    if pesel_str.len() < 11 {
        let mut new_value_str = "0".to_string();

        for _ in (pesel_str.len()+1)..11 {
            new_value_str.push('0');
        }

        pesel_str = new_value_str + &pesel_str;
    }

    if date_of_birth(pesel).is_none() { return Err(ValidationError::BirthDate); }

    let mut sum = 0;
    for (i, digit) in pesel_str.chars().take(11).map(|char| char.to_digit(10).unwrap()).enumerate() {
        sum += (digit as u8) * PESEL_WEIGHTS[i];
    }

    if let Some(Some(last_digit)) = sum.to_string().chars().last().map(|char| char.to_digit(10)) {
        if last_digit != 0 { return Err(ValidationError::ControlDigit); }
        Ok(())
    } else {
        Err(ValidationError::ControlDigit)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
#[error("{0}")]
pub enum PeselTryFromError<T> {
    ValidationError(#[from] ValidationError),
    Other(T),
}

#[macro_export]
macro_rules! impl_try_from_str_for_pesel {
    ($name:ident) => {
        impl TryFrom<&str> for $name {
            type Error = PeselTryFromError<std::num::ParseIntError>;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                let value = u64::from_str_radix(value, 10).map_err(PeselTryFromError::Other)?;
                validate(value)?;
                Self::try_from(value).map_err(PeselTryFromError::ValidationError)
            }
        }

        impl TryFrom<&String> for $name {
            type Error = PeselTryFromError<std::num::ParseIntError>;

            fn try_from(value: &String) -> Result<Self, Self::Error> {
                Self::try_from(value.as_str())
            }
        }

        impl TryFrom<String> for $name {
            type Error = PeselTryFromError<std::num::ParseIntError>;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::try_from(&value)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    static PESEL1: u64 = 02290486168;
    static PESEL2: u64 = 01302534699;
    static PESEL3: u64 = 00010128545;
    static PESEL4: u64 = 98250993285;
    static PESEL5: u64 = 60032417874;

    #[test]
    fn day_section() {
        assert_eq!(super::day_section(PESEL1), 04);
        assert_eq!(super::day_section(PESEL2), 25);
        assert_eq!(super::day_section(PESEL3), 01);
        assert_eq!(super::day_section(PESEL4), 09);
        assert_eq!(super::day_section(PESEL5), 24);
    }

    #[test]
    fn month_section() {
        assert_eq!(super::month_section(PESEL1), 29);
        assert_eq!(super::month_section(PESEL2), 30);
        assert_eq!(super::month_section(PESEL3), 01);
        assert_eq!(super::month_section(PESEL4), 25);
        assert_eq!(super::month_section(PESEL5), 03);
    }

    #[test]
    fn year_section() {
        assert_eq!(super::year_section(PESEL1), 02);
        assert_eq!(super::year_section(PESEL2), 01);
        assert_eq!(super::year_section(PESEL3), 00);
        assert_eq!(super::year_section(PESEL4), 98);
        assert_eq!(super::year_section(PESEL5), 60);
    }

    #[test]
    fn ordinal_section() {
        assert_eq!(super::ordinal_section(PESEL1), 8616);
        assert_eq!(super::ordinal_section(PESEL2), 3469);
        assert_eq!(super::ordinal_section(PESEL3), 2854);
        assert_eq!(super::ordinal_section(PESEL4), 9328);
        assert_eq!(super::ordinal_section(PESEL5), 1787);
    }

    #[test]
    fn control_section() {
        assert_eq!(super::control_section(PESEL1), 8);
        assert_eq!(super::control_section(PESEL2), 9);
        assert_eq!(super::control_section(PESEL3), 5);
        assert_eq!(super::control_section(PESEL4), 5);
        assert_eq!(super::control_section(PESEL5), 4);
    }

    #[test]
    fn day() {
        assert_eq!(super::day(PESEL1), 04);
        assert_eq!(super::day(PESEL2), 25);
        assert_eq!(super::day(PESEL3), 01);
        assert_eq!(super::day(PESEL4), 09);
        assert_eq!(super::day(PESEL5), 24);
    }

    #[test]
    fn month() {
        assert_eq!(super::month(PESEL1), Some(09));
        assert_eq!(super::month(PESEL2), Some(10));
        assert_eq!(super::month(PESEL3), Some(01));
        assert_eq!(super::month(PESEL4), Some(05));
        assert_eq!(super::month(PESEL5), Some(03));
    }

    #[test]
    fn invalid_month() {
        assert_eq!(super::month(02990486168u64), None);
        assert_eq!(super::month(02970486168u64), None);
        assert_eq!(super::month(02930486168u64), None);
    }

    #[test]
    fn year() {
        assert_eq!(super::year(PESEL1), 2002);
        assert_eq!(super::year(PESEL2), 2001);
        assert_eq!(super::year(PESEL3), 1900);
        assert_eq!(super::year(PESEL4), 2098);
        assert_eq!(super::year(PESEL5), 1960);
    }

    #[test]
    fn date_of_birth() {
        assert_eq!(super::date_of_birth(PESEL1), NaiveDate::from_ymd_opt(2002, 09, 04));
        assert_eq!(super::date_of_birth(PESEL2), NaiveDate::from_ymd_opt(2001, 10, 25));
        assert_eq!(super::date_of_birth(PESEL3), NaiveDate::from_ymd_opt(1900, 01, 01));
        assert_eq!(super::date_of_birth(PESEL4), NaiveDate::from_ymd_opt(2098, 05, 09));
        assert_eq!(super::date_of_birth(PESEL5), NaiveDate::from_ymd_opt(1960, 03, 24));
    }

    #[test]
    fn gender() {
        assert_eq!(super::gender(PESEL1), Gender::Female);
        assert_eq!(super::gender(PESEL2), Gender::Male);
        assert_eq!(super::gender(PESEL3), Gender::Female);
        assert_eq!(super::gender(PESEL4), Gender::Female);
        assert_eq!(super::gender(PESEL5), Gender::Male);
    }

    #[test]
    fn validate() {
        assert_eq!(super::validate(PESEL1), Ok(()));
        assert_eq!(super::validate(PESEL2), Ok(()));
        assert_eq!(super::validate(PESEL3), Ok(()));
        assert_eq!(super::validate(PESEL4), Ok(()));
        assert_eq!(super::validate(PESEL5), Ok(()));
    }

    #[test]
    fn invalid_pesels() {
        assert_eq!(super::validate(4355u64), Err(ValidationError::TooShort(4)));
        assert_eq!(super::validate(435585930294485u64), Err(ValidationError::TooLong(15)));
        assert_eq!(super::validate(99990486167u64), Err(ValidationError::BirthDate));
        assert_eq!(super::validate(02290486167u64), Err(ValidationError::ControlDigit));
    }
}

