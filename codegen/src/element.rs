#![allow(unused)] // the structs are designed to mimic the definition
use crate::buffer::Str;

pub struct Protocol {
    pub name: Str,
    pub copyright: Option<Str>,
    pub description: Option<Description>,
}

pub struct Description {
    /// in the `.dtd` file, summary is required
    /// but in the book, its optional
    pub summary: Str,
    pub content: Str,
}

pub struct Interface {
    pub name: Str,
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
    pub name: Str,
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
    pub name: Str,
    pub ty: Type,
    pub interface: Option<Str>,
    pub allow_null: bool,
    pub enum_name: Option<Str>,
    pub summary: Option<Str>,
    pub description: Option<Description>,
}

pub struct Enum {
    pub name: Str,
    pub since: Option<u32>,
    pub bitfield: bool,
    pub entries: Vec<Entry>,
    pub description: Option<Description>,
}

pub struct Entry {
    pub name: Str,
    pub value: Str,
    pub since: Option<u32>,
    pub deprecated_since: Option<u32>,
    pub summary: Option<Str>,
    pub description: Option<Description>,
}
