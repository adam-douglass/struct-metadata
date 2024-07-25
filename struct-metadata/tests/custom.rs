#![cfg(test)]

use struct_metadata::{Kind, Described, Descriptor, Entry, MetadataKind};


#[derive(Default, PartialEq, Eq, Debug, MetadataKind)]
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

#[derive(Described)]
#[metadata_type(Properties, defaults: false)]
struct NoneFeatured;

#[test]
fn single_featured() {
    let data = SingleFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "SingleFeatured", children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, Properties{important: true, cats: ""});
}

#[test]
fn dual_featured() {
    let data = DoubleFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "DoubleFeatured", children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, Properties{important: true, cats: "Less than 10"});
}

#[test]
fn none_featured() {
    let data = NoneFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "NoneFeatured", children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, Properties{..Default::default()});
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
#[metadata(important: true, cats="Less than 10")]
struct DoubleVecFeatured;

#[test]
fn single_vec_featured() {
    let data = SingleVecFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "SingleVecFeatured", children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, vec![("important", "true")]);
}

#[test]
fn dual_vec_featured() {
    let data = DoubleVecFeatured::metadata();
    assert_eq!(data.kind, Kind::Struct{ name: "DoubleVecFeatured", children: vec![]});
    assert_eq!(data.docs, None);
    assert_eq!(data.metadata, vec![("important", "true"), ("cats", "\"Less than 10\"")]);
}


/// non trivial metadata structs
#[derive(Described)]
#[allow(unused)]
#[metadata_type(Properties)]
#[metadata(important: true)]
struct Fields {
    /// Name used
    label: u64,

    #[metadata(cats: "fluffy")]
    description: String,

    /// Are cats allowed here?
    #[metadata(important: true)]
    cats: bool,
}


fn expected_fields_metadata() -> Descriptor<Properties> {
    Descriptor {
        docs: Some(vec!["non trivial metadata structs"]),
        metadata: Properties { important: true, cats: "" },
        kind: Kind::Struct {
            name: "Fields",
            children: vec![
                Entry { label: "label", docs: Some(vec!["Name used"]), has_default: false, metadata: Default::default(), type_info: u64::metadata() },
                Entry { label: "description", docs: None, has_default: false, metadata: Properties { cats: "fluffy", ..Default::default() }, type_info: String::metadata() },
                Entry { label: "cats", docs: Some(vec!["Are cats allowed here?"]), has_default: false, metadata: Properties { important: true, cats: "" }, type_info: bool::metadata() },
            ]
        }
    }
}

#[test]
fn fields() {
    assert_eq!(Fields::metadata(), expected_fields_metadata());
}



/// nested structs
#[derive(Described)]
#[allow(unused)]
#[metadata_type(Properties)]
#[metadata(important: true)]
struct Nested {
    /// Name used
    label: u64,

    #[metadata(cats: "with stripes")]
    data: Fields
}

#[test]
fn nested() {
    assert_eq!(Nested::metadata(), Descriptor{
        docs: Some(vec!["nested structs"]),
        metadata: Properties { important: true, cats: "" },
        kind: Kind::Struct {
            name: "Nested",
            children: vec![
                Entry { label: "label", docs: Some(vec!["Name used"]), has_default: false, metadata: Default::default(), type_info: u64::metadata() },
                Entry { label: "data", docs: None, has_default: false, metadata: Properties { cats: "with stripes", ..Default::default() }, type_info: expected_fields_metadata() },
            ]
        }
    });
}

