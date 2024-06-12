use struct_metadata::{Described, Descriptor, Variant};
use struct_metadata_derive::DescribedEnumString;

#[derive(strum::Display, strum::EnumString, DescribedEnumString)]
#[strum(serialize_all = "lowercase")]
pub enum ExtendedScanValues {
    Submitted,
    Skipped,
    Incomplete,
    #[metadata(ideal=true)]
    Complete,
}

#[test]
fn enum_display() {
    assert_eq!(ExtendedScanValues::metadata(), Descriptor {
        docs: None,
        metadata: Default::default(),
        kind: struct_metadata::Kind::Enum {
            name: "ExtendedScanValues",
            variants: vec![
                Variant{ label: "submitted".to_owned(), docs: None, metadata: Default::default() },
                Variant{ label: "skipped".to_owned(), docs: None, metadata: Default::default() },
                Variant{ label: "incomplete".to_owned(), docs: None, metadata: Default::default() },
                Variant{ label: "complete".to_owned(), docs: None, metadata: [("ideal", "true")].into_iter().collect() },
            ]
        }
    })
}