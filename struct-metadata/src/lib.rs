


pub struct Descriptor {
    pub docs: Option<Vec<&'static str>>,
    pub name: String,
    pub kind: String,
}

pub trait Described {
    fn metadata() -> Descriptor;
}
