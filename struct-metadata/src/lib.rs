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
pub struct Descriptor<Metadata: Default=HashMap<&'static str, &'static str>> {
    pub docs: Option<Vec<&'static str>>,
    pub metadata: Metadata,
    pub kind: Kind<Metadata>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Kind<Metadata: Default=HashMap<&'static str, &'static str>> {
    Struct { name: &'static str, children: Vec<Entry<Metadata>>, },
    Aliased { name: &'static str, kind: Box<Descriptor<Metadata>> },
    Sequence( Box<Descriptor<Metadata>> ),
    Mapping( Box<Descriptor<Metadata>>, Box<Descriptor<Metadata>> ),
    Bool,
    U64,
    String,
}


#[derive(Debug)]
pub struct Entry<Metadata: Default=HashMap<&'static str, &'static str>> {
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

impl<M: Default, T: Described<M>> Described<M> for Vec<T> {
    fn metadata() -> Descriptor<M> {
        Descriptor { 
            docs: None, 
            metadata: Default::default(), 
            kind: Kind::Sequence(Box::new(T::metadata())) 
        }
    }
}

impl<M: Default, K: Described<M> + core::hash::Hash, V: Described<M>> Described<M> for HashMap<K, V> {
    fn metadata() -> Descriptor<M> {
        Descriptor { 
            docs: None, 
            metadata: Default::default(), 
            kind: Kind::Mapping(Box::new(K::metadata()), Box::new(V::metadata())) 
        }
    }
}
