#![cfg(test)]

use std::collections::VecDeque;

use struct_metadata::{Described, Descriptor, Kind, Entry};

/// non trivial metadata structs
#[derive(Described)]
#[allow(unused)]
struct OptionVec {
    /// Name used
    label: Option<String>,

    #[metadata(active=true)]
    score: Option<u64>,

    #[metadata(active=false)]
    attached: Vec<u64>,

    /// A queue of strings
    #[metadata(active=true)]
    queue: VecDeque<String>,
}


#[test]
fn option_vec() {
    assert_eq!(OptionVec::metadata(), Descriptor {
        docs: Some(vec!["non trivial metadata structs"]),
        metadata: Default::default(),
        kind: Kind::Struct {
            name: "OptionVec",
            children: vec![
                Entry { label: "label", docs: Some(vec!["Name used"]), metadata: Default::default(), has_default: false, type_info: Descriptor { docs: None, metadata: Default::default(), kind: Kind::Option(Box::new(String::metadata())) }, aliases: &["label"] },
                Entry { label: "score", docs: None, metadata: [("active", "true")].into_iter().collect(), has_default: false, type_info: Descriptor { docs: None, metadata: Default::default(), kind: Kind::Option(Box::new(u64::metadata())) }, aliases: &["score"] },
                Entry { label: "attached", docs: None, metadata: [("active", "false")].into_iter().collect(), has_default: false, type_info: Descriptor { docs: None, metadata: Default::default(), kind: Kind::Sequence(Box::new(u64::metadata())) }, aliases: &["attached"] },
                Entry { label: "queue", docs: Some(vec!["A queue of strings"]), metadata: [("active", "true")].into_iter().collect(), has_default: false, type_info: Descriptor { docs: None, metadata: Default::default(), kind: Kind::Sequence(Box::new(String::metadata())) }, aliases: &["queue"] },
            ]
        }
    });
}

