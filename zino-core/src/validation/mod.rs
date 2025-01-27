//! Generic validator and common validation rules.
use crate::{error::Error, extension::JsonObjectExt, Map, SharedString};

mod validator;

pub use validator::{
    AlphabeticValidator, AlphanumericValidator, AsciiAlphabeticValidator,
    AsciiAlphanumericValidator, AsciiDigitValidator, AsciiHexdigitValidator,
    AsciiLowercaseValidator, AsciiUppercaseValidator, AsciiValidator, DateTimeValidator,
    DateValidator, HostValidator, HostnameValidator, IpAddrValidator, Ipv4AddrValidator,
    Ipv6AddrValidator, LowercaseValidator, NumericValidator, TimeValidator, UppercaseValidator,
    UriValidator, UuidValidator, Validator,
};

#[cfg(feature = "validator-credit-card")]
pub use validator::CreditCardValidator;

#[cfg(feature = "validator-email")]
pub use validator::EmailValidator;

#[cfg(feature = "validator-phone-number")]
pub use validator::PhoneNumberValidator;

#[cfg(feature = "validator-regex")]
pub use validator::RegexValidator;

/// A record of validation results.
#[derive(Debug, Default)]
pub struct Validation {
    failed_entries: Vec<(SharedString, Error)>,
}

impl Validation {
    /// Creates a new instance.
    #[inline]
    pub fn new() -> Self {
        Self {
            failed_entries: Vec::new(),
        }
    }

    /// Creates a new instance with the entry.
    #[inline]
    pub fn from_entry(key: impl Into<SharedString>, err: impl Into<Error>) -> Self {
        let failed_entries = vec![(key.into(), err.into())];
        Self { failed_entries }
    }

    /// Records an entry with the supplied message.
    #[inline]
    pub fn record(&mut self, key: impl Into<SharedString>, message: impl Into<SharedString>) {
        self.failed_entries.push((key.into(), Error::new(message)));
    }

    /// Records an entry for the error.
    #[inline]
    pub fn record_fail(&mut self, key: impl Into<SharedString>, err: impl Into<Error>) {
        self.failed_entries.push((key.into(), err.into()));
    }

    /// Validates the string value with a specific format.
    pub fn validate_format(&mut self, key: impl Into<SharedString>, value: &str, format: &str) {
        match format {
            "alphabetic" => {
                if let Err(err) = AlphabeticValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "alphanumeric" => {
                if let Err(err) = AlphanumericValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii" => {
                if let Err(err) = AsciiValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii-alphabetic" => {
                if let Err(err) = AsciiAlphabeticValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii-alphanumeric" => {
                if let Err(err) = AsciiAlphanumericValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii-digit" => {
                if let Err(err) = AsciiDigitValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii-hexdigit" => {
                if let Err(err) = AsciiHexdigitValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii-lowercase" => {
                if let Err(err) = AsciiLowercaseValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ascii-uppercase" => {
                if let Err(err) = AsciiUppercaseValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            #[cfg(feature = "validator-credit-card")]
            "credit-card" => {
                if let Err(err) = CreditCardValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "date" => {
                if let Err(err) = DateValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "date-time" => {
                if let Err(err) = DateTimeValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            #[cfg(feature = "validator-email")]
            "email" => {
                if let Err(err) = EmailValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "host" => {
                if let Err(err) = HostValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "hostname" => {
                if let Err(err) = HostnameValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ip" => {
                if let Err(err) = IpAddrValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ipv4" => {
                if let Err(err) = Ipv4AddrValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "ipv6" => {
                if let Err(err) = Ipv6AddrValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "lowercase" => {
                if let Err(err) = LowercaseValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "numeric" => {
                if let Err(err) = NumericValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            #[cfg(feature = "validator-phone-number")]
            "phone_number" => {
                if let Err(err) = PhoneNumberValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            #[cfg(feature = "validator-regex")]
            "regex" => {
                if let Err(err) = RegexValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "time" => {
                if let Err(err) = TimeValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "uppercase" => {
                if let Err(err) = UppercaseValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "uri" => {
                if let Err(err) = UriValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            "uuid" => {
                if let Err(err) = UuidValidator.validate(value) {
                    self.record_fail(key, err);
                }
            }
            _ => {
                tracing::warn!("supported format `{format}`");
            }
        }
    }

    /// Returns true if the validation contains a value for the specified key.
    #[inline]
    pub fn contains_key(&self, key: &str) -> bool {
        self.failed_entries.iter().any(|(field, _)| field == key)
    }

    /// Returns `true` if the validation is success.
    #[inline]
    pub fn is_success(&self) -> bool {
        self.failed_entries.is_empty()
    }

    /// Consumes the validation and returns as a json object.
    #[must_use]
    pub fn into_map(self) -> Map {
        let failed_entries = self.failed_entries;
        let mut map = Map::with_capacity(failed_entries.len());
        for (key, err) in failed_entries {
            map.upsert(key, err.to_string());
        }
        map
    }
}
