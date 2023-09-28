//! Macros for attaching metadata to structs.
//!
//! When a struct is used to define an interface with external systems there is
//! often additonal information about the external system that is required.
//! Rather than having that information stored separately this library
//! intends to maintain a single source of truth.

#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

pub use struct_metadata_derive::Described;

use std::collections::HashMap;

/// Information about a type along with its metadata and doc-strings.
#[derive(Debug, PartialEq, Eq)]
pub struct Descriptor<Metadata: Default> {
    /// Docstring for the type
    pub docs: Option<Vec<&'static str>>,
    /// Metadata for the type
    pub metadata: Metadata,
    /// Details about the type
    pub kind: Kind<Metadata>,
}

/// Enum reflecting all supported types
#[derive(Debug, PartialEq, Eq)]
pub enum Kind<Metadata: Default> {
    /// The type is a struct
    Struct {
        /// Name given to the struct in its declaration
        name: &'static str,
        /// List of fields within this struct
        children: Vec<Entry<Metadata>>,
    },
    /// A struct wrapping a single anonymous field
    Aliased {
        /// Name given to the struct in its declaration
        name: &'static str,
        /// The type this alias struct wraps
        kind: Box<Descriptor<Metadata>>
    },
    /// A simple no-field enum type
    Enum {
        /// Name given to the enum in its declaration
        name: &'static str,
        /// Information about each variant value within this enum
        variants: Vec<Variant<Metadata>>,
    },
    /// A list of items of a consistent type
    Sequence( Box<Descriptor<Metadata>> ),
    /// An item which is optionally present
    Option( Box<Descriptor<Metadata>> ),
    /// A pairwise mapping between consistent types with unique keys
    Mapping( Box<Descriptor<Metadata>>, Box<Descriptor<Metadata>> ),
    /// A field describing a point in time
    DateTime,
    /// A string
    String,
    /// Unsigned 128 bit integer
    U128,
    /// Signed 128 bit integer
    I128,
    /// Unsigned 64 bit integer
    U64,
    /// Signed 64 bit integer
    I64,
    /// Unsigned 32 bit integer
    U32,
    /// Signed 32 bit integer
    I32,
    /// Unsigned 16 bit integer
    U16,
    /// Signed 16 bit integer
    I16,
    /// Unsigned 8 bit integer
    U8,
    /// Signed 8 bit integer
    I8,
    /// 64 bit floating point number
    F64,
    /// 32 bit floating point number
    F32,
    /// A boolean value
    Bool,
    /// A value of unspecified type
    Any,
}

/// Struct describing an enum variant
#[derive(Debug, PartialEq, Eq)]
pub struct Variant<Metadata: Default> {
    /// String value used to describe the variant
    /// This respects strum's renaming policy
    pub label: String,
    /// doc strings describing this variant
    pub docs: Option<Vec<&'static str>>,
    /// metadata describing this variant
    pub metadata: Metadata,
}

/// Struct describing a struct field
#[derive(Debug)]
pub struct Entry<Metadata: Default> {
    /// Label of the field in question
    /// This respects serde's rename attribute
    pub label: &'static str,
    /// doc string describing this field
    pub docs: Option<Vec<&'static str>>,
    /// metadata describing this field
    pub metadata: Metadata,
    /// Type of this field
    pub type_info: Descriptor<Metadata>,
}

impl<T: PartialEq + Default> PartialEq for Entry<T> {
    fn eq(&self, other: &Self) -> bool {
        self.label == other.label && self.docs == other.docs && self.metadata == other.metadata && self.type_info == other.type_info
    }
}

impl<T: Eq + Default> Eq for Entry<T> {}

/// A self description of the type being targeted including doc-strings and metadata annotations.
pub trait Described<M: Default=HashMap<&'static str, &'static str>> {
    /// Get self description of this type
    fn metadata() -> Descriptor<M>;
}

/// Generate the simple formulaic implementation of Described for a basic type
macro_rules! basic_described {
    ($type_name:ident, $type_macro:ident) => {
        impl<M: Default> Described<M> for $type_name {
            fn metadata() -> Descriptor<M> { Descriptor { docs: None, metadata: M::default(), kind: Kind::$type_macro } }
        }
    };
}

basic_described!{String, String}
basic_described!{i128, I128}
basic_described!{u128, U128}
basic_described!{i64, I64}
basic_described!{u64, U64}
basic_described!{i32, I32}
basic_described!{u32, U32}
basic_described!{i16, I16}
basic_described!{u16, U16}
basic_described!{i8, I8}
basic_described!{u8, U8}
basic_described!{f64, F64}
basic_described!{f32, F32}
basic_described!{bool, Bool}


impl<M: Default, T: Described<M>> Described<M> for Option<T> {
    fn metadata() -> Descriptor<M> {
        Descriptor {
            docs: None,
            metadata: M::default(),
            kind: Kind::Option(Box::new(T::metadata()))
        }
    }
}

#[cfg(feature = "std")]
impl<M: Default, T: Described<M>> Described<M> for Box<T> {
    fn metadata() -> Descriptor<M> { T::metadata() }
}

#[cfg(feature = "std")]
impl<M: Default, T: Described<M>> Described<M> for Vec<T> {
    fn metadata() -> Descriptor<M> {
        Descriptor {
            docs: None,
            metadata: M::default(),
            kind: Kind::Sequence(Box::new(T::metadata()))
        }
    }
}

#[cfg(feature = "std")]
impl<M: Default, K: Described<M> + core::hash::Hash, V: Described<M>> Described<M> for HashMap<K, V> {
    fn metadata() -> Descriptor<M> {
        Descriptor {
            docs: None,
            metadata: M::default(),
            kind: Kind::Mapping(Box::new(K::metadata()), Box::new(V::metadata()))
        }
    }
}

#[cfg(feature = "chrono")]
impl<M: Default, Tz: chrono::TimeZone> Described<M> for chrono::DateTime<Tz> {
    fn metadata() -> Descriptor<M> {
        Descriptor { docs: None, metadata: M::default(), kind: Kind::DateTime }
    }
}

#[cfg(feature = "serde_json")]
impl<M: Default> Described<M> for serde_json::Value {
    fn metadata() -> Descriptor<M> {
        Descriptor { docs: None, metadata: M::default(), kind: Kind::Any }
    }
}

#[cfg(feature = "serde_json")]
impl<M: Default, K: Described<M>, V: Described<M>> Described<M> for serde_json::Map<K, V> {
    fn metadata() -> Descriptor<M> {
        Descriptor {
            docs: None,
            metadata: M::default(),
            kind: Kind::Mapping(Box::new(K::metadata()), Box::new(V::metadata()))
        }
    }
}