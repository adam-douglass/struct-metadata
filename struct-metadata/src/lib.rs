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
pub struct Descriptor<Metadata=HashMap<&'static str, &'static str>> {
    pub docs: Option<Vec<&'static str>>,
    pub metadata: Metadata,
    pub kind: Kind,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Struct { name: &'static str, children: Vec<Entry>, },
    Aliased { name: &'static str, kind: Box<Descriptor> },
    Bool,
    U64,
    String,
}


#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    pub label: &'static str,
    pub docs: Option<Vec<&'static str>>,
    pub metadata: HashMap<&'static str, &'static str>,
    pub type_info: Descriptor,
}



pub trait Described<M: Default=HashMap<&'static str, &'static str>> {
    fn metadata() -> Descriptor<M>;
}

impl<M: Default> Described<M> for bool {
    fn metadata() -> Descriptor<M> {
        Descriptor { docs: None, metadata: Default::default(), kind: Kind::Bool }
    }
}

impl<M: Default> Described<M> for u64 {
    fn metadata() -> Descriptor<M> {
        Descriptor { docs: None, metadata: Default::default(), kind: Kind::U64 }
    }
}

impl<M: Default> Described<M> for String {
    fn metadata() -> Descriptor<M> {
        Descriptor { docs: None, metadata: Default::default(), kind: Kind::String }
    }
}
