//! Where a one-time code or a message goes: a phone number (SMS) or an email address.

use std::fmt;

use crate::{auth::phone::Phone, error::ApiError};

/// A lower-cased email address with a plausible shape. Whether it exists is proved by the
/// one-time code sent to it, not by parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Email(String);

impl Email {
    pub fn parse(input: &str) -> Option<Self> {
        let email = input.trim().to_lowercase();
        let (local, domain) = email.split_once('@')?;
        let valid = email.len() <= 254
            && !local.is_empty()
            && local.len() <= 64
            && !domain.contains('@')
            && domain.contains('.')
            && !domain.starts_with('.')
            && !domain.ends_with('.')
            && domain.rsplit('.').next().is_some_and(|tld| tld.len() >= 2)
            && !email.chars().any(|c| c.is_whitespace() || c.is_control());
        valid.then_some(Self(email))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// The address with most of the local part hidden, safe for logs: `b***@gmail.com`.
    pub fn masked(&self) -> String {
        let (local, domain) = self.0.split_once('@').unwrap_or((&self.0, ""));
        let first = local.chars().next().unwrap_or('*');
        format!("{first}***@{domain}")
    }
}

impl fmt::Display for Email {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    Phone(Phone),
    Email(Email),
}

impl Destination {
    /// Exactly one of `phone` and `email`, validated.
    pub fn from_request(phone: Option<&str>, email: Option<&str>) -> Result<Self, ApiError> {
        match (phone, email) {
            (Some(phone), None) => {
                Phone::parse(phone)
                    .map(Self::Phone)
                    .ok_or_else(|| ApiError::BadRequest {
                        code: "invalid_phone",
                        message: "Enter a Nigerian mobile number, e.g. 0803 123 4567.".to_owned(),
                    })
            }
            (None, Some(email)) => {
                Email::parse(email)
                    .map(Self::Email)
                    .ok_or_else(|| ApiError::BadRequest {
                        code: "invalid_email",
                        message: "Enter a valid email address.".to_owned(),
                    })
            }
            _ => Err(ApiError::BadRequest {
                code: "invalid_body",
                message: "Send either phone or email.".to_owned(),
            }),
        }
    }

    /// Stored in `sign_in_codes.channel`.
    pub fn channel(&self) -> &'static str {
        match self {
            Self::Phone(_) => "sms",
            Self::Email(_) => "email",
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Phone(p) => p.as_str(),
            Self::Email(e) => e.as_str(),
        }
    }

    pub fn masked(&self) -> String {
        match self {
            Self::Phone(p) => p.masked(),
            Self::Email(e) => e.masked(),
        }
    }

    pub fn phone(&self) -> Option<&str> {
        match self {
            Self::Phone(p) => Some(p.as_str()),
            Self::Email(_) => None,
        }
    }

    pub fn email(&self) -> Option<&str> {
        match self {
            Self::Email(e) => Some(e.as_str()),
            Self::Phone(_) => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emails_are_trimmed_and_lower_cased() {
        let email = Email::parse("  Ada.Obi@Example.COM ").unwrap();
        assert_eq!(email.as_str(), "ada.obi@example.com");
        assert_eq!(email.masked(), "a***@example.com");
    }

    #[test]
    fn malformed_emails_are_rejected() {
        for input in [
            "",
            "ada",
            "@example.com",
            "ada@",
            "ada@example",
            "ada@.com",
            "ada@example.",
            "ada@example.c",
            "a@b@example.com",
            "ada obi@example.com",
        ] {
            assert!(
                Email::parse(input).is_none(),
                "{input:?} should be rejected"
            );
        }
    }

    #[test]
    fn exactly_one_of_phone_and_email_is_required() {
        assert!(matches!(
            Destination::from_request(Some("08031234567"), None),
            Ok(Destination::Phone(_))
        ));
        assert!(matches!(
            Destination::from_request(None, Some("ada@example.com")),
            Ok(Destination::Email(_))
        ));
        assert!(Destination::from_request(None, None).is_err());
        assert!(Destination::from_request(Some("08031234567"), Some("ada@example.com")).is_err());
    }
}
