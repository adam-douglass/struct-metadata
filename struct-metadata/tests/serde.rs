use serde::{Serialize, Deserialize};
use struct_metadata::{Described, Descriptor, Kind, Entry};


#[derive(Serialize, Deserialize, Described, Debug, PartialEq, Eq)]
#[allow(dead_code)]
struct RenameField {
    #[serde(rename = "type")]
    field_type: String,
}

#[test]
fn rename_field() {
    assert_eq!(serde_json::to_string(&RenameField{field_type: "abc".to_owned()}).unwrap(), "{\"type\":\"abc\"}");
    assert_eq!(serde_json::from_str::<RenameField>("{\"type\": \"abc\"}").unwrap(), RenameField{field_type: "abc".to_owned()});
    assert_eq!(RenameField::metadata(), Descriptor {
        docs: None,
        metadata: Default::default(),
        kind: Kind::Struct {
            name: "RenameField",
            children: vec![
                Entry { label: "type", docs: None, metadata: Default::default(), type_info: String::metadata() }
            ]
        }
    });
}

#[test]
fn rename_variant() {
    panic!();
}

#[test]
fn rename_all() {
    panic!();
}


#[test]
fn rename_all_fields() {
    panic!();
}


#[test]
fn rename_struct() {
    panic!();
}

