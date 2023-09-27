//! Macros for attaching metadata to structs.
//!
//! When a struct is used to define an interface with external systems there is
//! often additonal information about the external system that is required.
//! Rather than having that information stored separately this library
//! intends to maintain a single source of truth.

#![warn(missing_docs, non_ascii_idents, trivial_numeric_casts,
    unused_crate_dependencies, noop_method_call, single_use_lifetimes, trivial_casts,
    unused_lifetimes, nonstandard_style, variant_size_differences)]
#![deny(keyword_idents)]
#![warn(clippy::missing_docs_in_private_items)]
#![allow(clippy::needless_return, clippy::while_let_on_iterator)]

pub use struct_metadata_derive::Described;

use std::collections::HashMap;

/// Information about a type along with its metadata and docstrings.
#[derive(Debug, PartialEq, Eq)]
pub struct Descriptor<Metadata: Default> {
    pub docs: Option<Vec<&'static str>>,
    pub metadata: Metadata,
    pub kind: Kind<Metadata>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind<Metadata: Default> {
    Struct { name: &'static str, children: Vec<Entry<Metadata>>, },
    Aliased { name: &'static str, kind: Box<Descriptor<Metadata>> },
    Enum { name: &'static str, variants: Vec<Variant<Metadata>>, },
    Sequence( Box<Descriptor<Metadata>> ),
    Option( Box<Descriptor<Metadata>> ),
    Mapping( Box<Descriptor<Metadata>>, Box<Descriptor<Metadata>> ),
    DateTime,
    String,
    U64,
    I64,
    U32,
    I32,
    U16,
    I16,
    U8,
    I8,
    F64,
    F32,
    Bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Variant<Metadata: Default> {
    pub label: String,
    pub docs: Option<Vec<&'static str>>,
    pub metadata: Metadata,
}


#[derive(Debug)]
pub struct Entry<Metadata: Default> {
    pub label: &'static str,
    pub docs: Option<Vec<&'static str>>,
    pub metadata: Metadata,
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
