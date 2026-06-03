#![allow(unused)]
use std::num::NonZeroU32;

use crate::str::Str;

#[derive(Debug)]
pub struct Protocol {
    pub name: Str,
    pub copyright: Option<Str>,
    pub desc: Option<Description>,
    pub interfaces: Vec<Interface>,
}

#[derive(Debug)]
pub struct Interface {
    pub name: Str,
    pub version: u32,
    pub frozen: Option<bool>,
    pub desc: Option<Description>,
    pub items: Vec<Item>,
}

#[derive(Debug)]
pub enum Item {
    Operation(Operation),
    Enum(Enum),
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum OpKind {
    Request,
    Event,
}

#[derive(Debug)]
pub struct Operation {
    pub kind: OpKind,
    pub name: Str,
    pub ty: Option<Str>,
    pub since: Option<NonZeroU32>,
    pub dep_since: Option<NonZeroU32>,
    pub desc: Option<Description>,
    pub args: Vec<Arg>,
}

#[derive(Debug)]
pub struct Enum {
    pub name: Str,
    pub since: Option<NonZeroU32>,
    pub bitfield: Option<bool>,
    pub desc: Option<Description>,
    pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Arg {
    pub name: Str,
    pub ty: Str,
    pub summary: Option<Str>,
    pub interface: Option<Str>,
    pub allow_null: Option<bool>,
    pub enum_: Option<Str>,
    pub desc: Option<Description>,
}

#[derive(Debug)]
pub struct Entry {
    pub name: Str,
    pub value: Str,
    pub summary: Option<Str>,
    pub since: Option<NonZeroU32>,
    pub dep_since: Option<NonZeroU32>,
    pub desc: Option<Description>,
}

#[derive(Debug)]
pub struct Description {
    pub summary: Str,
    pub content: Str,
}
