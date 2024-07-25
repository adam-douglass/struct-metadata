use struct_metadata::{Described, Kind};


#[allow(dead_code)]
#[derive(serde::Deserialize, Described)]
struct FieldDefaults {
    #[serde(default)]
    has_default: u64,
    #[serde(default="make_number")]
    also_has_default: u64,
    no_default: u64,
}

fn make_number() -> u64 { 10 }

#[allow(dead_code)]
#[derive(serde::Deserialize, Described, Default)]
#[serde(default)]
struct StructDefault {
    #[serde(default)]
    double_default: u64,
    has_default: u64,
}


#[test]
fn default_defined() {
    let data = FieldDefaults::metadata();
    let Kind::Struct{ name, children} = data.kind else { panic!() };
    assert_eq!(name, "FieldDefaults");
    
    assert_eq!(children[0].label, "has_default");
    assert!(children[0].has_default);

    assert_eq!(children[1].label, "also_has_default");
    assert!(children[1].has_default);

    assert_eq!(children[2].label, "no_default");
    assert!(!children[2].has_default);


    let data = StructDefault::metadata();
    let Kind::Struct{ name, children} = data.kind else { panic!() };
    assert_eq!(name, "StructDefault");
    
    assert_eq!(children[0].label, "double_default");
    assert!(children[0].has_default);

    assert_eq!(children[1].label, "has_default");
    assert!(children[1].has_default);
}
