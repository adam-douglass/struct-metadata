#![cfg(test)]

use struct_metadata::{Kind, Described};


#[derive(Default, PartialEq, Eq, Debug)]
struct Properties {
    pub important: bool,
    pub cats: &'static str,
}


#[derive(Described)]
#[metadata_type(Properties)]
#[metadata(important: true)]
struct SingleFeatured;

#[derive(Described)]
#[metadata_type(Properties, defaults: false)]
#[metadata(important: true, cats: "Less than 10")]
struct DoubleFeatured;

#[test]
fn single_featured() {
    let data = SingleFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "SingleFeatured".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, Properties{important: true, cats: ""});
}

#[test]
fn dual_featured() {
    let data = DoubleFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "DoubleFeatured".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, Properties{important: true, cats: "Less than 10"});
}

// This should cause a compiler error when uncommented
// #[derive(Described)]
// #[metadata_type(Properties)]
// #[metadata(dogs: true)]
// struct DogFeatured;


#[derive(Described)]
#[metadata_sequence(Vec<(&'static str, &'static str)>)]
#[metadata(important: true)]
struct SingleVecFeatured;

#[derive(Described)]
#[metadata_sequence(Vec<(&'static str, &'static str)>)]
#[metadata(important: true, cats: "Less than 10")]
struct DoubleVecFeatured;

#[test]
fn single_vec_featured() {
    let data = SingleVecFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "SingleVecFeatured".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, vec![("important", "true")]);
}

#[test]
fn dual_vec_featured() {
    let data = DoubleVecFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "DoubleVecFeatured".to_string(), children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, vec![("important", "true"), ("cats", "\"Less than 10\"")]);
}
