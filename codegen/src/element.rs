use crate::Bytes;

pub struct Protocol {
    pub name: Bytes,
    pub copyright: Option<Bytes>,
    pub description: Option<Description>,
}

#[allow(unused)]
pub struct Description {
    /// in the `.dtd` file, summary is required
    /// but in the book, its optional
    pub summary: Bytes,
    pub content: Bytes,
}

pub struct Interface {
    pub name: Bytes,
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

    pub fn as_str(&self) -> &'static str {
        match self {
            OpKind::Request => "request",
            OpKind::Event => "event",
        }
    }
}

pub struct Op {
    pub name: Bytes,
    pub description: Option<Description>,
    pub kind: OpKind,
    pub destructor: bool,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
    pub args: Vec<Arg>,
}

pub struct Arg {
    pub name: Bytes,
    pub description: Option<Description>,
    pub ty: Type,
    pub interface: Option<Bytes>,
    pub allow_null: bool,
    pub enum_name: Option<Bytes>,
    pub summary: Option<Bytes>,
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
    pub fn from_wl_type(ty: &[u8]) -> Self {
        match ty {
            b"int" => Self::Int,
            b"uint" => Self::Uint,
            b"fixed" => Self::Fixed,
            b"string" => Self::String,
            b"array" => Self::Array,
            b"fd" => Self::Fd,
            b"new_id" => Self::NewId,
            b"object" => Self::Object,
            _ => panic!("unknown type: {:?}", str::from_utf8(ty)),
        }
    }

    pub fn to_wl_type(&self) -> &'static str {
        match self {
            Self::Int => "int",
            Self::Uint => "uint",
            Self::Fixed => "fixed",
            Self::String => "string",
            Self::Array => "array",
            Self::Fd => "fd",
            Self::NewId => "new_id",
            Self::Object => "object",
        }
    }

    pub fn to_rust_type(&self, inferred_interface: bool) -> &'static str {
        match self {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => "&'a str",
            Type::Array => "&'a [u8]",
            Type::Fd => "RawFd",
            Type::NewId => {
                if inferred_interface {
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
    pub name: Bytes,
    pub description: Option<Description>,
    pub since: Option<u32>,
    pub bitfield: bool,
    pub entries: Vec<Entry>,
}

pub struct Entry {
    pub name: Bytes,
    pub value: Bytes,
    pub summary: Option<Bytes>,
    pub description: Option<Description>,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
}
