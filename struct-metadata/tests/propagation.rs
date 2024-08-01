
#![cfg(test)]

use struct_metadata::{Described, Kind, MetadataKind};

#[derive(Default, PartialEq, Eq, Debug)]
struct Meta {
    index: Option<bool>,
}

impl MetadataKind for Meta {
    fn forward_propagate_entry_defaults(&mut self, context: &Self, kind: &Self) {
        self.index = self.index.or(kind.index).or(context.index)
    }
}


#[derive(Described)]
#[metadata_type(Meta)]
#[allow(unused)]
struct Test1 {
    #[metadata()]
    default: String,
    #[metadata(index=true)]
    indexed: String,
    #[metadata(index=false)]
    not_indexed: String,
    #[metadata()]
    odefault: Option<String>,
    #[metadata(index=true)]
    oindexed: Option<String>,
    #[metadata(index=false)]
    onot_indexed: Option<String>,
}

#[derive(Described)]
#[metadata_type(Meta)]
#[metadata(index=true)]
#[allow(unused)]
struct Test2 {
    #[metadata]
    default: String,
    #[metadata(index=true)]
    indexed: String,
    #[metadata(index=false)]
    not_indexed: String,
}


#[derive(Described)]
#[metadata_type(Meta)]
#[metadata(index=false)]
#[allow(unused)]
struct Test3 {
    default: String,
    #[metadata(index=true)]
    indexed: String,
    #[metadata(index=false)]
    not_indexed: String,
}

#[test]
fn test_index_defaults() {

    let meta = Test1::metadata();
    let Kind::Struct { name, children } = meta.kind else { panic!() };
    assert_eq!(name, "Test1");
    assert_eq!(children[0].metadata, Meta{ index: None });
    assert_eq!(children[1].metadata, Meta{ index: Some(true) });
    assert_eq!(children[2].metadata, Meta{ index: Some(false) });
    assert_eq!(children[3].metadata, Meta{ index: None });
    assert_eq!(children[4].metadata, Meta{ index: Some(true) });
    assert_eq!(children[5].metadata, Meta{ index: Some(false) });
    
    let meta = Test2::metadata();
    let Kind::Struct { name, children } = meta.kind else { panic!() };
    assert_eq!(name, "Test2");
    assert_eq!(children[0].metadata, Meta{ index: Some(true) });
    assert_eq!(children[1].metadata, Meta{ index: Some(true) });
    assert_eq!(children[2].metadata, Meta{ index: Some(false) });

    let meta = Test3::metadata();
    let Kind::Struct { name, children } = meta.kind else { panic!() };
    assert_eq!(name, "Test3");
    assert_eq!(children[0].metadata, Meta{ index: Some(false) });
    assert_eq!(children[1].metadata, Meta{ index: Some(true) });
    assert_eq!(children[2].metadata, Meta{ index: Some(false) });
}

#[derive(Described)]
#[metadata_type(Meta)]
#[allow(unused)]
struct SubModel {
    default: String,
    #[metadata(index=true)]
    indexed: String,
    #[metadata(index=false)]
    not_indexed: String,
}

#[derive(Described)]
#[metadata_type(Meta)]
#[allow(unused)]
struct Test4 {
    default: SubModel,
    #[metadata(index=true)]
    indexed: SubModel,
    #[metadata(index=false)]
    not_indexed: SubModel, 
}

#[test]
fn test_compound_index_defaults() {
    let meta = Test4::metadata();
    let Kind::Struct { name, children } = meta.kind else { panic!() };
    assert_eq!(name, "Test4");

    assert_eq!(children[0].label, "default");
    if let Kind::Struct { name, children } = &children[0].type_info.kind {
        assert_eq!(*name, "SubModel");
        assert_eq!(children[0].label, "default");
        assert_eq!(children[0].metadata, Meta{ index: None });
        assert_eq!(children[1].label, "indexed");
        assert_eq!(children[1].metadata, Meta{ index: Some(true) });
        assert_eq!(children[2].label, "not_indexed");
        assert_eq!(children[2].metadata, Meta{ index: Some(false) });
    } else { panic!() }

    assert_eq!(children[1].label, "indexed");
    if let Kind::Struct { name, children } = &children[1].type_info.kind {
        assert_eq!(*name, "SubModel");
        assert_eq!(children[0].label, "default");
        assert_eq!(children[0].metadata, Meta{ index: Some(true) });
        assert_eq!(children[1].label, "indexed");
        assert_eq!(children[1].metadata, Meta{ index: Some(true) });
        assert_eq!(children[2].label, "not_indexed");
        assert_eq!(children[2].metadata, Meta{ index: Some(false) });
    } else { panic!() }

    assert_eq!(children[2].label, "not_indexed");
    if let Kind::Struct { name, children } = &children[2].type_info.kind {
        assert_eq!(*name, "SubModel");
        assert_eq!(children[0].label, "default");
        assert_eq!(children[0].metadata, Meta{ index: Some(false) });
        assert_eq!(children[1].label, "indexed");
        assert_eq!(children[1].metadata, Meta{ index: Some(true) });
        assert_eq!(children[2].label, "not_indexed");
        assert_eq!(children[2].metadata, Meta{ index: Some(false) });
    } else { panic!() }

}