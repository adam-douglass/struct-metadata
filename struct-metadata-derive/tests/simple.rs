#![cfg(test)]

use struct_metadata::{Described, Kind, Entry};


#[derive(Described)]
struct EmptyA;

#[derive(Described)]
struct EmptyB {}

#[test]
fn empty_a() {
    let data = EmptyA::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "EmptyA".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert!(data.metadata.is_empty());
}

#[test]
fn empty_b() {
    let data = EmptyB::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "EmptyB".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert!(data.metadata.is_empty());
}


#[derive(Described)]
/// Docstring
struct EmptyDocA;

#[derive(Described)]
/// The
///
/// Docstring
struct EmptyDocB {}

#[test]
fn empty_doc_a() {
    let data = EmptyDocA::metadata();
    assert_eq!(data.kind, Kind::Struct{name: "EmptyDocA".to_string(), children: vec![]});
    assert_eq!(data.docs, Some(vec!["Docstring"]));
    assert!(data.metadata.is_empty());
}

#[test]
fn empty_doc_b() {
    let data = EmptyDocB::metadata();
    assert_eq!(data.kind, Kind::Struct{name: "EmptyDocB".to_string(), children: vec![]});
    assert_eq!(data.docs, Some(vec!["The", "", "Docstring"]));
    assert!(data.metadata.is_empty());
}


#[derive(Described)]
struct Single(u64);

#[test]
fn single() {
    let data = Single::metadata();
    assert_eq!(data.kind, Kind::Aliased { name: "Single".to_string(), kind: Box::new(u64::metadata()) });
    assert_eq!(data.docs, None);
    assert!(data.metadata.is_empty());
}

#[derive(Described)]
#[metadata(important: true)]
struct SingleFeatured;

#[derive(Described)]
#[metadata(important: true, cats: "Less than 10")]
struct DoubleFeatured;

#[test]
fn single_featured() {
    let data = SingleFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "SingleFeatured".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, [("important", "true")].into_iter().collect());
}

#[test]
fn dual_featured() {
    let data = DoubleFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "DoubleFeatured".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, [("important", "true"), ("cats", "\"Less than 10\"")].into_iter().collect());
}

#[derive(Described)]
#[metadata(important: true)]
#[allow(dead_code)]
struct SimpleFields {
    /// Name used
    label: u64,

    #[metadata(text: true)]
    description: String,

    /// Are cats allowed here?
    #[metadata(important: true)]
    cats: bool,
}

#[test]
fn simple_fields() {
    let data = SimpleFields::metadata();
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, [("important", "true")].into_iter().collect());
    assert_eq!(data.kind, Kind::Struct{ name: "SimpleFields".to_string(), children: vec![
        Entry { label: "label".to_string(), docs: Some(vec!["Name used"]), metadata: Default::default(), type_info: u64::metadata() },
        Entry { label: "description".to_string(), docs: None, metadata: [("text", "true")].into_iter().collect(), type_info: String::metadata() },
        Entry { label: "cats".to_string(), docs: Some(vec!["Are cats allowed here?"]), metadata: [("important", "true")].into_iter().collect(), type_info: bool::metadata() },
    ]});
}
