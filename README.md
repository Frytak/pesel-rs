[PESEL](https://en.wikipedia.org/wiki/PESEL) validation and detail extraction with multiple data layout implementations.

[![Crates.io Version](https://img.shields.io/crates/v/pesel-rs?color=green)](https://crates.io/crates/pesel-rs)
[![Static Badge](https://img.shields.io/badge/docs-orange)](https://docs.rs/pesel-rs/latest/pesel_rs/)
[![Crates.io License](https://img.shields.io/crates/l/pesel-rs)](https://crates.io/crates/pesel-rs)

# Definitions

PESEL: `YYMMDDOOOOC`

- `YY` - Last two digits of year of birth
- `MM` - Month of birth (shifted depending on year of birth as shown by the table below)

| Year        | 1800 - 1899 | 1900 - 1999 | 2000 - 2099 | 2100 - 2199 | 2200 - 2299 |
|-------------|-------------|-------------|-------------|-------------|-------------|
| Month shift | +80         | 0           | +20         | +40         | +60         |

- `DD` - Day of birth
- `OOOO` - Ordinal number, where the last digit denotes the gender ([0, 2, 4, 6, 8] = female, [1, 3, 5, 7, 9] = male)
- `C` - Control number

# Usage

There are two PESEL structs provided by the crate, both implementing the [`PeselTrait`](https://docs.rs/pesel-rs/latest/pesel_rs/trait.PeselTrait.html).

- [`crate::bit_fields::Pesel`](https://docs.rs/pesel-rs/latest/pesel_rs/bit_fields/struct.Pesel.html) - Stores each section of the PESEL in the following layout: `7 bits | YY | 5 bits | MM | 5 bits | DD | 5 bits | OOOO | 5 bits | C`, where in between bits are unused. Extracting each field is done using bitwise operations. You can get the human readable number using `u64::from`.

- [`crate::human_redable::Pesel`](https://docs.rs/pesel-rs/latest/pesel_rs/human_redable/struct.Pesel.html) - Stores the PESEL as a plain number, extracting each field requires modulo and division operations, if often accessing individual fields is important to you, you should probably use [`crate::bit_fields::Pesel`](https://docs.rs/pesel-rs/latest/pesel_rs/bit_fields/struct.Pesel.html).

If you just need to validate a number or extract a specific section without using the structs, you could use functions in the lib root. Most of these functions won't check if the value they're returning is valid, unlike the structs who are guaranteed to always return a valid value.

# Examples

Function that takes a name and welcomes the person based on date of birth and gender from the PESEL. Implemented using [`crate::bit_fields::Pesel`](https://docs.rs/pesel-rs/latest/pesel_rs/bit_fields/struct.Pesel.html) because we're mostly reading the fields.

```rust
use pesel_rs::{prelude::*, bit_fields::Pesel};

fn welcome(first_name: &str, pesel: u64) {
    match Pesel::try_from(pesel) {
        Ok(pesel) => {
            if pesel.date_of_birth() > NaiveDate::from_ymd_opt(2015, 1, 1).unwrap() {
                let gender = if pesel.gender() == Gender::Male { "boy" } else { "girl" };
                println!("Wow {first_name}! You're such a young {gender}!");
            } else {
                println!("{first_name}, you're very old, I'm sorry 😞");
            }
        }
        Err(_) => println!("Huh, what you gave me doesn't seem to be a valid pesel {first_name}..."),
    }
}
```

Function finding a pesel with the oldest date of birth. Working with a generic PESEL, we introduce additional bounds (required by [`PeselTrait`](https://docs.rs/pesel-rs/latest/pesel_rs/trait.PeselTrait.html)).
```rust
use pesel_rs::prelude::*;

fn oldest<T: PeselTrait>(pesels: &[T])
where
    u64: From<T>,
    for<'a> u64: From<&'a T>
{
    assert!(pesels.len() > 0);

    let mut oldest_index = 0;
    pesels.iter().skip(1).enumerate().for_each(|(i, pesel)| {
        if pesels[oldest_index].date_of_birth() < pesel.date_of_birth() {
            oldest_index = i;
        }
    });

    let date_of_birth = pesels[oldest_index].date_of_birth();
    println!("PESEL nr. {oldest_index} is the oldest! Born at {date_of_birth}")
}
```
