[PESEL](https://en.wikipedia.org/wiki/PESEL) validation and detail extraction with multiple data layout implementations.

There is already the [pesel](https://docs.rs/pesel/latest/pesel/index.html) crate, you may want to check it out, but I've found it's implementation suboptimal for my case.

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

There are two PESEL structs provided by the crate, both implementing the `PeselTrait`.

- `crate::bit_fields::Pesel` - Stores each section of the PESEL in the following layout: `7 bits | YY | 5 bits | MM | 5 bits | DD | 5 bits | OOOO | 5 bits | C`, where in between bits are unused. Extracting each field is done using bitwise operations. You can get the human readable number using `u64::from`.

- `crate::human_redable::Pesel` - Stores the PESEL as a plain number, extracting each field requires modulo and division operations, if often accessing individual fields is important to you, you should probably use `crate::bit_fields::Pesel`.

If you just need to validate a number or extract a specific section without using the structs, you could use functions in the lib root. Most of these functions won't check if the value they're returning is valid, unlike the structs who are guaranteed to always return a valid value.
