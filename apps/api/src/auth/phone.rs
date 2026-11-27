//! Phone numbers. Every served state is in Nigeria, so numbers are normalised to Nigerian
//! E.164 (`+234` and ten digits, starting with 7, 8 or 9).

use std::fmt;

/// A phone number in E.164 form, e.g. `+2348031234567`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Phone(String);

impl Phone {
    /// Accepts the common ways Nigerians write a mobile number: `08031234567`, `8031234567`,
    /// `2348031234567` and `+234 803 123 4567`. Spaces, dashes, dots and brackets are ignored.
    pub fn parse(input: &str) -> Option<Self> {
        let (plus, rest) = match input.trim().strip_prefix('+') {
            Some(rest) => (true, rest),
            None => (false, input.trim()),
        };
        let mut digits = String::with_capacity(rest.len());
        for c in rest.chars() {
            match c {
                '0'..='9' => digits.push(c),
                ' ' | '-' | '.' | '(' | ')' => {}
                _ => return None,
            }
        }
        let national = match digits.len() {
            13 if digits.starts_with("234") => &digits[3..],
            11 if !plus && digits.starts_with('0') => &digits[1..],
            10 if !plus => &digits[..],
            _ => return None,
        };
        if !national.starts_with(['7', '8', '9']) {
            return None;
        }
        Some(Self(format!("+234{national}")))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The number with all but the last four digits hidden, safe for logs.
    pub fn masked(&self) -> String {
        let tail = &self.0[self.0.len().saturating_sub(4)..];
        format!("+234******{tail}")
    }
}

impl fmt::Display for Phone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::Phone;

    #[test]
    fn common_formats_normalise_to_e164() {
        for input in [
            "08031234567",
            "8031234567",
            "2348031234567",
            "+2348031234567",
            "+234 803 123 4567",
            " 0803-123-4567 ",
            "(0803) 123.4567",
        ] {
            assert_eq!(
                Phone::parse(input).map(|p| p.to_string()).as_deref(),
                Some("+2348031234567"),
                "{input}"
            );
        }
    }

    #[test]
    fn invalid_numbers_are_rejected() {
        for input in [
            "",
            "0803123456",      // too short
            "080312345678",    // too long
            "06031234567",     // not a mobile prefix
            "+08031234567",    // plus with a trunk zero
            "+4478031234567",  // not Nigerian
            "0803123456a",     // letters
            "+234803123456*7", // symbols
        ] {
            assert!(
                Phone::parse(input).is_none(),
                "{input:?} should be rejected"
            );
        }
    }

    #[test]
    fn masked_keeps_only_the_last_four_digits() {
        let phone = Phone::parse("08031234567").unwrap();
        assert_eq!(phone.masked(), "+234******4567");
    }
}
