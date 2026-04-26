use crate::String;

pub struct Protocol {
    pub name: String,
    pub copyright: Option<String>,
    pub description: Option<Description>,
}

pub struct Description {
    pub summary: Option<String>,
    pub content: String,
}

pub struct Interface {
    pub name: String,
    pub description: Option<Description>,
    pub version: u32,
    pub frozen: bool,
}

pub enum OpKind {
    Request,
    Event,
}

impl OpKind {
    pub fn is_request(&self) -> bool {
        matches!(self, Self::Request)
    }

    pub fn is_event(&self) -> bool {
        matches!(self, Self::Event)
    }
}

pub struct Op {
    pub name: String,
    pub description: Option<Description>,
    pub kind: OpKind,
    pub destructor: bool,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
    pub args: Vec<Arg>,
}

pub struct Arg {
    pub name: String,
    pub description: Option<Description>,
    pub ty: Type,
    pub interface: Option<String>,
    pub allow_null: bool,
    pub enum_name: Option<String>,
    pub summary: Option<String>,
}

impl Arg {
    pub fn need_lifetime(args: &[Arg]) -> bool {
        args.iter()
            .any(|arg| matches!(arg.ty, Type::String | Type::Array))
    }
}

pub enum Type {
    Int,
    Uint,
    Fixed,
    String,
    Array,
    Fd,
    NewId,
    Object,
}

impl Type {
    pub fn to_rust_type(&self, interface: Option<&str>) -> &'static str {
        match self {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => "&'a str",
            Type::Array => "Array<'a>",
            Type::Fd => "RawFd",
            Type::NewId => {
                if interface.is_some() {
                    "u32"
                } else {
                    "NewId"
                }
            }
            Type::Object => "u32",
        }
    }
}

pub struct Enum {
    pub name: String,
    pub description: Option<Description>,
    pub since: Option<u32>,
    pub bitfield: bool,
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub name: String,
    pub value: String,
    pub summary: Option<String>,
    pub description: Option<Description>,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
}
