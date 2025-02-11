
#![cfg(test)]

use serde::{Deserialize, Serialize};
use struct_metadata::{Described, Kind, MetadataKind};

#[derive(Default, PartialEq, Eq, Debug)]
struct Meta {
    index: Option<bool>,
}

impl MetadataKind for Meta {
    fn forward_propagate_context(&mut self, context: &Self) {
        self.index = self.index.or(context.index)
    }

    fn forward_propagate_child_defaults(&mut self, kind: &Self) {
        self.index = self.index.or(kind.index)
    }

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
fn index_defaults() {

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
fn compound_index_defaults() {
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

#[derive(Serialize, Deserialize, Described)]
#[metadata_type(Meta)]
struct InlineInner {
    a: u32,
}

#[derive(Serialize, Deserialize, Described)]
#[metadata_type(Meta)]
#[allow(dead_code)]
struct InlineOuterMetadata {
    #[metadata(index=true)]
    #[serde(flatten)]
    inner: InlineInner,
    #[metadata(index=false)]
    other: i64,
    other2: i64,
}

#[test]
fn flattened_metadata() {
    let Kind::Struct {children, ..} = InlineOuterMetadata::metadata().kind else { panic!() };
    assert_eq!(children.len(), 3);
    for child in children {
        match child.label {
            "other" => assert_eq!(child.metadata, Meta { index: Some(false) }),
            "other2" => assert_eq!(child.metadata, Meta { index: None }),
            "a" => assert_eq!(child.metadata, Meta { index: Some(true) }),
            _ => panic!(),
        }
    }
}


#[derive(Described)]
#[metadata_type(Meta)]
#[allow(unused)]
struct Test5 {
    default: Option<SubModel>,
    #[metadata(index=true)]
    indexed: Option<SubModel>,
    #[metadata(index=false)]
    not_indexed: Option<SubModel>, 
}

#[test]
fn compound_index_defaults_with_option() {
    let meta = Test5::metadata();
    let Kind::Struct { name, children } = meta.kind else { panic!() };
    assert_eq!(name, "Test5");

    assert_eq!(children[0].label, "default");
    if let Kind::Option(inner) = &children[0].type_info.kind {
        if let Kind::Struct { name, children } = &inner.kind {
            assert_eq!(*name, "SubModel");
            assert_eq!(children[0].label, "default");
            assert_eq!(children[0].metadata, Meta{ index: None });
            assert_eq!(children[1].label, "indexed");
            assert_eq!(children[1].metadata, Meta{ index: Some(true) });
            assert_eq!(children[2].label, "not_indexed");
            assert_eq!(children[2].metadata, Meta{ index: Some(false) });
        } else { panic!() }
    } else { panic!() }

    assert_eq!(children[1].label, "indexed");
    if let Kind::Option(inner) = &children[1].type_info.kind {
        if let Kind::Struct { name, children } = &inner.kind {
            assert_eq!(*name, "SubModel");
            assert_eq!(children[0].label, "default");
            assert_eq!(children[0].metadata, Meta{ index: Some(true) });
            assert_eq!(children[1].label, "indexed");
            assert_eq!(children[1].metadata, Meta{ index: Some(true) });
            assert_eq!(children[2].label, "not_indexed");
            assert_eq!(children[2].metadata, Meta{ index: Some(false) });
        } else { panic!() }
    } else { panic!() }

    assert_eq!(children[2].label, "not_indexed");
    if let Kind::Option(inner) = &children[2].type_info.kind {
        if let Kind::Struct { name, children } = &inner.kind {
            assert_eq!(*name, "SubModel");
            assert_eq!(children[0].label, "default");
            assert_eq!(children[0].metadata, Meta{ index: Some(false) });
            assert_eq!(children[1].label, "indexed");
            assert_eq!(children[1].metadata, Meta{ index: Some(true) });
            assert_eq!(children[2].label, "not_indexed");
            assert_eq!(children[2].metadata, Meta{ index: Some(false) });
        } else { panic!() }
    } else { panic!() }
}

// #[derive(Described)]
// #[metadata_type(Meta)]
// #[allow(unused)]
// struct Inner1 {
//     indexed_number: i32
// }

// #[derive(Described)]
// #[metadata_type(Meta)]
// #[allow(unused)]
// struct Inner2 {
//     indexed: Option<Inner1>
// }

// #[derive(Described)]
// #[metadata_type(Meta)]
// #[allow(unused)]
// #[metadata(index=true)]
// struct Outer {
//     indexed: Option<Inner2>
// }

// #[test]
// fn layered_propagation() {
//     let meta = Outer::metadata();
//     assert!(meta.metadata.index.unwrap());
//     let Kind::Struct { children, .. } = meta.kind else { panic!() };
//     assert!(children[0].metadata.index.unwrap());
//     let Kind::Option(ref kind) = children[0].type_info.kind else { panic!() };
//     let Kind::Struct{ref children, ..} = kind.kind else { panic!() };
//     assert!(children[0].metadata.index.unwrap());
//     let Kind::Option(ref kind) = children[0].type_info.kind else { panic!() };
//     let Kind::Struct{ref children, ..} = kind.kind else { panic!() };
//     assert_eq!(children[0].label, "indexed_number");
//     assert!(children[0].metadata.index.unwrap());
    
// }