//! A trait for a type that should be compatable with serde Deserialize but requires validation to be enforced recursively.

#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

pub use validation_boilerplate_derive::ValidatedDeserialize;
pub use serde::{Deserialize, Deserializer};

/// Trait for types that can be produced by validating input that can be deserialized normally.
/// Parameter type is the type of the validator.
pub trait ValidatedDeserialize<'de, Validator>: Sized {
    /// Type that can be deserialized as input to this type
    type ProxyType: Deserialize<'de>;

    /// Construct a type from prepared input and a validator class
    fn validate(input: Self::ProxyType, validator: &Validator) -> Result<Self, String>;

    /// Shortcut function for calling deserialize and validate back to back
    fn deserialize_and_validate<D>(deserializer: D, validator: &Validator) -> Result<Self, D::Error> where D: Deserializer<'de> {
        let proxy = Self::ProxyType::deserialize(deserializer)?;
        Self::validate(proxy, validator).map_err(serde::de::Error::custom)
    }
}

impl<'de, T, V> ValidatedDeserialize<'de, V> for Vec<T>
    where T: ValidatedDeserialize<'de, V>
{
    type ProxyType = Vec<<T as ValidatedDeserialize<'de, V>>::ProxyType>;
    
    fn validate(input: Self::ProxyType, validator: &V) -> Result<Self, String> {
        input.into_iter().map(|item| T::validate(item, validator)).collect()
    }
}
