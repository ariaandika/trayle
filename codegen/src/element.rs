#![allow(unused)] // the structs are designed to mimic the definition
use crate::Bytes;

pub struct Protocol {
    pub name: Bytes,
    pub copyright: Option<Bytes>,
    pub description: Option<Description>,
}

pub struct Description {
    /// in the `.dtd` file, summary is required
    /// but in the book, its optional
    pub summary: Bytes,
    pub content: Bytes,
}

pub struct Interface {
    pub name: Bytes,
    pub version: u32,
    pub frozen: bool,
    pub description: Option<Description>,
}

pub enum OpKind {
    Request,
    Event,
}

pub struct Op {
    pub kind: OpKind,
    pub name: Bytes,
    pub destructor: bool,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
    pub args: Vec<Arg>,
    pub description: Option<Description>,
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

pub struct Arg {
    pub name: Bytes,
    pub ty: Type,
    pub interface: Option<Bytes>,
    pub allow_null: bool,
    pub enum_name: Option<Bytes>,
    pub summary: Option<Bytes>,
    pub description: Option<Description>,
}

pub struct Enum {
    pub name: Bytes,
    pub since: Option<u32>,
    pub bitfield: bool,
    pub entries: Vec<Entry>,
    pub description: Option<Description>,
}

pub struct Entry {
    pub name: Bytes,
    pub value: Bytes,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
    pub summary: Option<Bytes>,
    pub description: Option<Description>,
}
