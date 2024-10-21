//! Każda z 11 cyfr w numerze PESEL ma swoje znaczenie. Można je podzielić następująco:
//! 
//! RRMMDDPPPPK
//! 
//! RR – to 2 ostanie cyfry roku urodzenia,
//! 
//! MM – to miesiąc urodzenia (zapoznaj się z sekcją  "Dlaczego osoby urodzone po 1999 roku mają inne oznaczenie miesiąca urodzenia", która znajduje się poniżej),
//! 
//! DD – to dzień urodzenia,
//! 
//! PPPP – to liczba porządkowa oznaczająca płeć. U kobiety ostatnia cyfra tej liczby jest parzysta (0, 2, 4, 6, 8), a u mężczyzny - nieparzysta (1, 3, 5, 7, 9),
//! 
//! K – to cyfra kontrolna.
//! 
//! Przykład: PESEL 810203PPP6K należy do kobiety, która urodziła się 3 lutego 1981 roku, a PESEL 761115PPP3K - do mężczyzny, który urodził się 15 listopada 1976 roku.
//!
//!
//! Aby odróżnić od siebie numery PESEL z różnych stuleci, przyjęto następującą metodę oznaczania miesiąca urodzenia:
//! 
//!  
//! 
//! Miesiąc	Stulecie	 	 	 	 
//!  	1800-1899	1900-1999	2000-2099	2100-2199	2200-2299
//! Styczeń	    81	01	21	41	61
//! Luty	    82	02	22	42	62
//! Marzec	    83	03	23	43	63
//! Kwiecień	84	04	24	44	64
//! Maj	        85	05	25	45	65
//! Czerwiec	86	06	26	46	66
//! Lipiec	    87	07	27	47	67
//! Sierpień	88	08	28	48	68
//! Wrzesień	89	09	29	49	69
//! Październik	90	10	30	50	70
//! Listopad	91	11	31	51	71
//! Grudzień	92	12	32	52	72

use chrono::NaiveDate;
use thiserror::Error;

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

pub trait PeselTrait: Into<u64> + TryFrom<u64> {
    /// RRMM`DD`PPPPK
    fn day_section(&self) -> u8;

    /// RR`MM`DDPPPPK
    fn month_section(&self) -> u8;

    /// `RR`MMDDPPPPK
    fn year_section(&self) -> u8;

    /// RRMMDD`PPPP`K
    fn ordinal_section(&self) -> u16;

    /// RRMMDDPPPP`K`
    fn control_section(&self) -> u8;

    fn day(&self) -> u8 {
        self.day_section()
    }

    fn month(&self) -> u8 {
        match month_from_section(self.month_section()) {
            Some(some) => some,
            None => unreachable!(),
        }
    }

    fn year(&self) -> u16 {
        year_from_sections(self.month_section(), self.year_section())
    }

    fn date_of_birth(&self) -> Option<NaiveDate> {
        let year = self.year() as i32;
        let month = self.month() as u32;
        let day = self.day() as u32;
        NaiveDate::from_ymd_opt(year, month, day)
    }
    
    fn sex(&self) -> Sex {
        if self.ordinal_section() % 2 == 0 {
            Sex::Female
        } else {
            Sex::Male
        }
    }
}

pub fn validate<P>(pesel: P) -> Result<(), ()>
where P: PeselTrait + Clone {
    let value: u64 = pesel.clone().into();
    let value_str = value.to_string();

    if value_str.len() < 8 {
        return Err(())
    }

    // TODO: Pad with zero to 11 
    if 11 > value_str.len() {
    }

    if pesel.date_of_birth().is_none() { return Err(()); }

    let mut sum = 0;
    for (i, digit) in value_str.chars().take(11).map(|char| char.to_digit(10).unwrap()).enumerate() {
        sum += (digit as u8) * PESEL_WEIGHTS[i];
    }

    Ok(())
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Pesel(u64);

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Sex {
    Male,
    Female,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Error)]
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

impl Pesel {
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

    pub const fn day_u64(value: u64) -> u8 {
        Self::day_section_u64(value)
    }

    /// # Errors
    /// Returns `None` if:
    /// - `month_section` is not in range of `<1,92>`
    pub const fn month_u64(value: u64) -> Option<u8> {
        Self::month_from_section(Self::month_section_u64(value))
    }

    pub const fn year_u64(value: u64) -> u16 {
        Self::year_from_sections(Self::month_section_u64(value), Self::year_section_u64(value))
    }

    pub const fn sex_u64(value: u64) -> Sex {
        if Self::ordinal_section_u64(value) % 2 == 0 {
            Sex::Female
        } else {
            Sex::Male
        }
    }

    /// RRMM`DD`PPPPK
    pub const fn day_section_u64(value: u64) -> u8 {
        ((value % 10_000_000) / 100_000) as u8
    }

    /// RR`MM`DDPPPPK
    pub const fn month_section_u64(value: u64) -> u8 {
        ((value % 1_000_000_000) / 10_000_000) as u8
    }

    /// `RR`MMDDPPPPK
    pub const fn year_section_u64(value: u64) -> u8 {
        ((value % 100_000_000_000) / 1_000_000_000) as u8
    }

    /// RRMMDD`PPPP`K
    pub const fn ordinal_section_u64(value: u64) -> u16 {
        ((value % 100_000) / 10) as u16
    }

    /// RRMMDDPPPP`K`
    pub const fn control_section_u64(value: u64) -> u8 {
        (value % 10) as u8
    }

    pub const fn date_of_birth_u64(value: u64) -> Option<NaiveDate> {
        let year = Self::year_u64(value) as i32;
        let month = match Self::month_u64(value) {
            Some(month) => month as u32,
            None => return None,
        };
        let day = Self::day_u64(value) as u32;
        NaiveDate::from_ymd_opt(year, month, day)
    }

    /// RRMMDDPPPPK
    pub fn validate_static<P>(value: P) -> Result<(), ValidationError>
    where P: AsRef<u64> {
        let value = value.as_ref();
        let mut value_str = value.to_string();

        if value_str.len() < 8 {
            return Err(ValidationError::TooShort(value_str.len()))
        }

        if value_str.len() > 11 {
            return Err(ValidationError::TooLong(value_str.len()))
        }

        // TODO: Pad with zero to 11 
        if value_str.len() < 11 {
            let mut new_value_str = "0".to_string();

            for _ in (value_str.len()+1)..11 {
                new_value_str.push('0');
            }

            value_str = new_value_str + &value_str;
        }

        if Self::date_of_birth_u64(*value).is_none() { return Err(ValidationError::BirthDate); }

        let mut sum = 0;
        for (i, digit) in value_str.chars().take(11).map(|char| char.to_digit(10).unwrap()).enumerate() {
            dbg!(i, digit, PESEL_WEIGHTS[i], (digit as u8) * PESEL_WEIGHTS[i]);
            sum += (digit as u8) * PESEL_WEIGHTS[i];
        }

        if let Some(Some(last_digit)) = sum.to_string().chars().last().map(|char| char.to_digit(10)) {
            if last_digit != 0 { return Err(ValidationError::ControlDigit); }
            Ok(())
        } else {
            return Err(ValidationError::ControlDigit);
        }
    }

    /// RRMM`DD`PPPPK
    pub const fn day_section(&self) -> u8 {
        Self::day_section_u64(self.0)
    }

    /// RR`MM`DDPPPPK
    pub const fn month_section(&self) -> u8 {
        Self::month_section_u64(self.0)
    }

    /// `RR`MMDDPPPPK
    pub const fn year_section(&self) -> u8 {
        Self::year_section_u64(self.0)
    }

    /// RRMMDD`PPPP`K
    pub const fn ordinal_section(&self) -> u16 {
        Self::ordinal_section_u64(self.0)
    }

    /// RRMMDDPPPP`K`
    pub const fn control_section(&self) -> u8 {
        Self::control_section_u64(self.0)
    }

    pub const fn day(&self) -> u8 {
        Self::day_u64(self.0)
    }

    pub const fn month(&self) -> u8 {
        match Self::month_u64(self.0) {
            Some(some) => some,
            None => unreachable!(),
        }
    }

    pub const fn year(&self) -> u16 {
        Self::year_from_sections(self.month_section(), self.year_section())
    }

    pub const fn date_of_birth(&self) -> Option<NaiveDate> {
        Self::date_of_birth_u64(self.0)
    }
    
    pub const fn sex(&self) -> Sex {
        Self::sex_u64(self.0)
    }

    pub const fn try_from_u64(value: u64) -> Result<Self, ()> {
        Ok(Self(value))
    }
}

impl AsRef<u64> for Pesel {
    fn as_ref(&self) -> &u64 {
        &self.0
    }
}

// TODO:
impl TryFrom<u64> for Pesel {
    type Error = ();

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Self::try_from_u64(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PESEL1: Pesel = match Pesel::try_from_u64(02290486168) { Ok(ok) => ok, Err(_) => panic!("Invalid pesel nr. 1"), };
    const PESEL2: Pesel = match Pesel::try_from_u64(08122888735) { Ok(ok) => ok, Err(_) => panic!("Invalid pesel nr. 2"), };
    const PESEL3: Pesel = match Pesel::try_from_u64(78920213443) { Ok(ok) => ok, Err(_) => panic!("Invalid pesel nr. 3"), };

    #[test]
    fn day_section() {
        assert_eq!(PESEL1.day_section(), 04);
        assert_eq!(PESEL2.day_section(), 28);
        assert_eq!(PESEL3.day_section(), 02);
    }

    #[test]
    fn month_section() {
        assert_eq!(PESEL1.month_section(), 29);
        assert_eq!(PESEL2.month_section(), 12);
        assert_eq!(PESEL3.month_section(), 92);
    }

    #[test]
    fn year_section() {
        assert_eq!(PESEL1.year_section(), 02);
        assert_eq!(PESEL2.year_section(), 08);
        assert_eq!(PESEL3.year_section(), 78);
    }

    #[test]
    fn ordinal_section() {
        assert_eq!(PESEL1.ordinal_section(), 8616);
        assert_eq!(PESEL2.ordinal_section(), 8873);
        assert_eq!(PESEL3.ordinal_section(), 1344);
    }

    #[test]
    fn control_section() {
        assert_eq!(PESEL1.control_section(), 8);
        assert_eq!(PESEL2.control_section(), 5);
        assert_eq!(PESEL3.control_section(), 3);
    }

    #[test]
    fn day() {
        assert_eq!(PESEL1.day(), 04);
        assert_eq!(PESEL2.day(), 28);
        assert_eq!(PESEL3.day(), 02);
    }

    #[test]
    fn month() {
        assert_eq!(PESEL1.month(), 09);
        assert_eq!(PESEL2.month(), 12);
        assert_eq!(PESEL3.month(), 12);
    }

    #[test]
    fn year() {
        assert_eq!(PESEL1.year(), 2002);
        assert_eq!(PESEL2.year(), 1908);
        assert_eq!(PESEL3.year(), 1878);
    }

    #[test]
    fn date_of_birth() {
        assert_eq!(PESEL1.date_of_birth(), NaiveDate::from_ymd_opt(2002, 09, 04));
        assert_eq!(PESEL2.date_of_birth(), NaiveDate::from_ymd_opt(1908, 12, 28));
        assert_eq!(PESEL3.date_of_birth(), NaiveDate::from_ymd_opt(1878, 12, 02));
    }

    #[test]
    fn sex() {
        assert_eq!(PESEL1.sex(), Sex::Female);
        assert_eq!(PESEL2.sex(), Sex::Male);
        assert_eq!(PESEL3.sex(), Sex::Female);
    }

    #[test]
    fn validate() {
        assert_eq!(Pesel::validate_static(PESEL1), Ok(()));
        assert_eq!(Pesel::validate_static(PESEL2), Err(ValidationError::ControlDigit));
        assert_eq!(Pesel::validate_static(PESEL3), Err(ValidationError::ControlDigit));
    }
}
