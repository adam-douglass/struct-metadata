

#[derive(Debug, PartialEq, Eq)]
pub enum Kind {
    Struct { children: Vec<(String, Descriptor)>, }
}



#[derive(Debug, PartialEq, Eq)]
pub struct Descriptor {
    pub docs: Option<Vec<&'static str>>,
    pub name: String,
    pub features: HashMap<String, String>,
    pub kind: Kind,
}

pub trait Described {
    fn metadata() -> Descriptor;
}
