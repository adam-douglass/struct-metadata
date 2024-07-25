use serde::{Serialize, Deserialize};
use struct_metadata::{Described, Descriptor, Entry, Kind, Variant};


#[derive(Serialize, Deserialize, Described, Debug, PartialEq, Eq)]
#[allow(dead_code)]
#[serde(rename = "RenameFieldTestType")]
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
            name: "RenameFieldTestType",
            children: vec![
                Entry { label: "type", docs: None, has_default: false, metadata: Default::default(), type_info: String::metadata(), aliases: &["type"] }
            ]
        }
    });
}

#[derive(Serialize, Deserialize, Described, Debug)]
#[allow(dead_code)]
enum RenameVarient {
    #[serde(rename = "type", alias="kind")]
    Type,
}

#[test]
fn rename_variant() {
    assert_eq!(RenameVarient::metadata(), Descriptor {
        docs: None,
        metadata: Default::default(),
        kind: struct_metadata::Kind::Enum {
            name: "RenameVarient",
            variants: vec![
                Variant{ label: "type", docs: None, metadata: Default::default(), aliases: &["type", "kind"] },
            ]
        }
    })
}

#[derive(Serialize, Deserialize, Described, Debug)]
#[allow(dead_code)]
#[serde(rename="OuterName", rename_all="UPPERCASE")]
enum RenameAllVarient {
    Type,
}


#[test]
fn rename_all_varients() {
    assert_eq!(RenameAllVarient::metadata(), Descriptor {
        docs: None,
        metadata: Default::default(),
        kind: struct_metadata::Kind::Enum {
            name: "OuterName",
            variants: vec![
                Variant{ label: "TYPE", docs: None, metadata: Default::default(), aliases: &["TYPE"] },
            ]
        }
    })
}

#[derive(Serialize, Deserialize, Described, Debug)]
#[allow(dead_code)]
struct RenameAllField {
    inner: u8,
}

#[test]
fn rename_all_fields() {
    assert_eq!(RenameAllField::metadata(), Descriptor {
        docs: None,
        metadata: Default::default(),
        kind: Kind::Struct {
            name: "RenameAllField",
            children: vec![
                Entry { label: "inner", docs: None, has_default: false, metadata: Default::default(), type_info: u8::metadata(), aliases: &["inner"] }
            ]
        }
    });
}


// #[test]
// fn rename_struct() {
//     panic!();
// }

#[derive(Serialize, Deserialize, Described)]
struct InlineInner {
    a: u32,
}

#[derive(Serialize, Deserialize, Described)]
#[allow(dead_code)]
struct InlineOuter {
    #[serde(flatten)]
    inner: InlineInner
}

#[test]
fn inline() {
    let Kind::Struct { name, children } = InlineInner::metadata().kind else { panic!() };
    assert_eq!(name, "InlineInner");
    let inner_children = children;
    let Kind::Struct { name, children } = InlineOuter::metadata().kind else { panic!() };
    assert_eq!(name, "InlineOuter");
    assert_eq!(inner_children, children);
}

