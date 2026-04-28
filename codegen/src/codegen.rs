use crate::Write;
use crate::element::*;

const P1: &str = "    ";
const P2: &str = "        ";
const P3: &str = "            ";
const P4: &str = "                ";

// prelude
const ENCODE_TRAIT: &str = "Encode";

const PRELUDE: &str = "\
#![allow(unused_imports)]
use std::num::NonZeroU32;
use std::os::fd::RawFd;

const fn roundup4(value: usize) -> usize {
    (value + 3) & (usize::MAX << 2)
}";

impl Protocol {
    pub fn generate_header(&self, o: &mut impl Write) {
        let name = &self.name;
        let cp = self.copyright.as_ref();

        writeln!(o, "//! {name}");
        writeln!(o, "//!");
        if let Some(cp) = cp {
            writeln!(o, "//! ===== COPYRIGHT =====");
            writeln!(o, "//!");
            for line in cp.as_str().lines().map(str::trim_start) {
                let sp = &" "[line.is_empty() as usize..];
                writeln!(o, "//!{sp}{line}");
            }
            writeln!(o, "//!");
            writeln!(o, "//! ===== COPYRIGHT =====");
        }
        writeln!(o, "{PRELUDE}");
    }
}

impl Interface {
    pub fn generate_header(&self, o: &mut impl Write) {
        let name = &self.name;
        let version = self.version;
        let frozen = self.frozen;
        writeln!(
            o,
            "\n\
            pub mod {name} {{\n\
            {P1}use super::*;\n\
            {P1}pub const VERSION: u32 = {version};\n\
            {P1}pub const FROZEN: bool = {frozen};"
        );
    }

    pub fn generate_trailer(o: &mut impl Write) {
        writeln!(o, "}}");
    }
}

impl Op {
    pub fn generate(&self, opcode: u32, o: &mut impl Write) {
        let destructor = self.destructor;
        let since = self.since;
        let deprecated_since = self.deprecated_since;
        let args = &self.args;
        let is_request = self.kind.is_request();
        let is_event = self.kind.is_event();
        let lifetime = if Arg::need_lifetime(args) { "<'a>" } else { "" };
        let mod_name = match &*self.name {
            b"move" => "r#move",
            _ => self.name.as_str(),
        };
        let struct_name = camel_case(&self.name);
        let full_name = format_args!("{mod_name}::{struct_name}");

        writeln!(o);

        // ===== constructor =====
        for arg in args {
            let name = &arg.name;
            let ty = arg.ty.to_wl_type();
            write!(o, "{P1}/// {name}: ");
            match &arg.enum_name {
                Some(enum_name) => write!(o, "{enum_name}"),
                None => {
                    write!(o, "{ty}");
                    if let Some(iface) = arg.interface.as_ref() {
                        write!(o, "<{iface}>");
                    }
                }
            }
            if arg.allow_null {
                write!(o, " | null");
            }
            writeln!(o);
        }
        writeln!(o, "{P1}#[inline]");
        write!(o, "{P1}pub const fn {mod_name}{lifetime}(");
        for (is_first, arg) in args.with_first() {
            let name = &arg.name;
            if !is_first {
                write!(o, ", ");
            }
            let ty = arg.to_rust_type();
            write!(o, "{name}: {ty}");
        }
        writeln!(o, ") -> {full_name}{lifetime} {{");
        write!(o, "{P2}{full_name} {{ ");
        for (is_first, arg) in args.with_first() {
            if !is_first {
                write!(o, ", ");
            }
            write!(o, "{}", arg.name);
        }
        writeln!(o, " }}");
        writeln!(o, "{P1}}}");
        writeln!(o);

        // ===== mod =====
        if let Some(since) = since {
            writeln!(o, "{P1}/// since: {since}");
        }
        if let Some(dep_since) = deprecated_since {
            writeln!(o, "{P1}/// deprecated-since: {dep_since}");
        }
        writeln!(o, "{P1}pub mod {mod_name} {{");
        writeln!(o, "{P2}use super::*;");
        writeln!(o, "{P2}pub const OPCODE: u32 = {opcode};");
        writeln!(o, "{P2}pub const IS_REQUEST: bool = {is_request};");
        writeln!(o, "{P2}pub const IS_EVENT: bool = {is_event};");
        writeln!(o, "{P2}pub const IS_DESTRUCTOR: bool = {destructor};");
        writeln!(o);

        // ===== struct =====
        write!(o, "{P2}pub struct {struct_name}{lifetime} {{");
        writeln!(o, "{}", &"}"[..args.is_empty() as usize]);
        for arg in args {
            let name = &arg.name;
            let rust_ty = arg.to_rust_type();
            let wl_ty = arg.ty.to_wl_type();

            write!(o, "{P3}/// type: ");
            match &arg.enum_name {
                Some(enum_name) => write!(o, "{enum_name}"),
                None => {
                    write!(o, "{wl_ty}");
                    if let Some(iface) = &arg.interface {
                        write!(o, "<{iface}>");
                    }
                }
            }
            writeln!(o);
            write!(o, "{P3}pub {name}: ");
            if arg.allow_null {
                writeln!(o, "Option<{rust_ty}>,");
            } else {
                writeln!(o, "{rust_ty},");
            }
        }
        if !args.is_empty() {
            writeln!(o, "{P2}}}");
        }
        writeln!(o);

        // ===== impl =====
        writeln!(o, "{P2}impl{lifetime} {struct_name}{lifetime} {{");

        // ===== fn size() =====
        let constant_size: u32 = args.iter().map(Arg::constant_size).sum();

        writeln!(o, "{P3}#[inline]");
        writeln!(o, "{P3}pub const fn size(&self) -> usize {{");
        write!(o, "{P4}{constant_size}");
        for arg in args {
            let Some(dyn_size) = arg.dynamic_size() else {
                continue;
            };
            write!(o, " + {dyn_size}");
        }
        writeln!(o, "\n{P3}}}");
        writeln!(o);

        // ===== fn copy_to_slice() =====
        writeln!(o, "{P3}pub fn copy_to_slice(&self, buf: &mut [u8]) {{");
        match args.as_slice() {
            [] => writeln!(o, "{P4}let _ = buf;"),
            [arg] => writeln!(o, "{P4}{ENCODE_TRAIT}::encode(&self.{}, buf);", arg.name),
            _ => {
                writeln!(o, "{P4}if buf.len() != self.size() {{");
                writeln!(
                    o,
                    "{P4}    panic!(\"buffer should have the exact required length\");"
                );
                writeln!(o, "{P4}}}");
                writeln!(o, "{P4}unsafe {{");
                for (is_last, arg) in args.with_last() {
                    let buf = if is_last {
                        "buf"
                    } else {
                        writeln!(
                            o,
                            "{P4}    let (write, buf) = buf.split_at_mut_unchecked({ENCODE_TRAIT}::size(&self.{}));",
                            arg.name
                        );
                        "write"
                    };
                    writeln!(
                        o,
                        "{P4}    {ENCODE_TRAIT}::encode_unchecked(&self.{}, {buf});",
                        arg.name
                    );
                }
                writeln!(o, "{P4}}}");
            }
        }
        writeln!(o, "{P3}}}");

        // ===== end impl =====
        writeln!(o, "{P2}}}");

        // ===== end mod =====
        writeln!(o, "{P1}}}");
    }
}

impl Enum {
    pub fn generate(&self, o: &mut impl Write) {
        let name = camel_case(&self.name);
        let bitfield = self.bitfield;
        let entries = &self.entries;

        writeln!(o);
        if let Some(since) = self.since {
            writeln!(o, "{P1}/// since: {since}");
        }
        writeln!(o, "{P1}/// bitfield: {bitfield}");
        writeln!(o, "{P1}pub enum {name} {{");
        for entry in entries {
            let name = camel_case(&entry.name);
            let value = &entry.value;
            if let Some(since) = entry.since {
                writeln!(o, "{P2}/// since: {since}");
            }
            if let Some(dep_since) = entry.deprecated_since {
                writeln!(o, "{P2}/// deprecated-since: {dep_since}");
            }
            writeln!(o, "{P2}{name} = {value},");
        }
        writeln!(o, "{P1}}}");
    }
}

impl Arg {
    fn need_lifetime(args: &[Arg]) -> bool {
        args.iter()
            .any(|arg| matches!(arg.ty, Type::String | Type::Array))
    }

    fn constant_size(&self) -> u32 {
        match self.ty {
            Type::Int | Type::Uint | Type::Fixed | Type::String | Type::Array | Type::Object => 4,
            Type::Fd => 0,
            Type::NewId => {
                if self.interface.is_some() {
                    4
                } else {
                    12
                }
            }
        }
    }

    fn dynamic_size<'a>(&'a self) -> Option<DynamicSize<'a>> {
        let is_null_term = match self.ty {
            Type::Int | Type::Uint | Type::Fixed | Type::Fd | Type::Object => return None,
            Type::String => true,
            Type::Array => false,
            Type::NewId => {
                if self.interface.is_some() {
                    return None;
                }
                true
            }
        };
        Some(DynamicSize {
            arg: self,
            is_null_term,
        })
    }

    fn to_rust_type(&self) -> &'static str {
        match self.ty {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => "&'a str",
            Type::Array => "&'a [u8]",
            Type::Fd => "RawFd",
            Type::NewId => {
                if self.interface.is_some() {
                    "u32"
                } else {
                    "NewId"
                }
            }
            Type::Object => "u32",
        }
    }
}

struct DynamicSize<'a> {
    arg: &'a Arg,
    is_null_term: bool,
}

impl<'a> std::fmt::Display for DynamicSize<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = if self.is_null_term { " + 1" } else { "" };
        write!(f, "roundup4(self.{}.len(){n})", self.arg.name.as_str())
    }
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

fn camel_case(bytes: &crate::Bytes) -> CamelCase<'_> {
    CamelCase { name: bytes.as_str() }
}

struct CamelCase<'a> {
    name: &'a str,
}

impl<'a> std::fmt::Display for CamelCase<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut chars = self.name.chars();
        let prefix = chars.next().expect("name should be non-empty");
        // some enum variant starts with digit
        if prefix.is_ascii_digit() {
            write!(f, "_")?;
        }
        write!(f, "{}", prefix.to_ascii_uppercase())?;
        while let Some(ch) = chars.next() {
            // wayland use snake case, rename to camel case
            let ch = if ch == '_' {
                let Some(next) = chars.next() else {
                    break;
                };
                next.to_ascii_uppercase()
            } else {
                ch
            };
            write!(f, "{ch}")?;
        }
        Ok(())
    }
}

trait IterExt: Sized {
    type Iter<'a>: Iterator where Self: 'a;

    fn with_first<'a>(&'a self) -> WithFirst<Self::Iter<'a>>;

    fn with_last<'a>(&'a self) -> WithLast<Self::Iter<'a>>;
}

impl<T> IterExt for Vec<T> {
    type Iter<'a>
        = std::slice::Iter<'a, T>
    where
        Self: 'a;

    fn with_first<'a>(&'a self) -> WithFirst<Self::Iter<'a>> {
        WithFirst {
            iter: self.iter(),
            is_first: true,
        }
    }

    fn with_last<'a>(&'a self) -> WithLast<Self::Iter<'a>> {
        match self.split_last() {
            Some((last, rest)) => WithLast {
                iter: rest.iter(),
                last: Some(last),
            },
            None => WithLast {
                iter: self.iter(),
                last: None,
            },
        }
    }
}

struct WithFirst<I> {
    iter: I,
    is_first: bool,
}

impl<I: Iterator> Iterator for WithFirst<I> {
    type Item = (bool, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        let is_first = self.is_first;
        let next = self.iter.next()?;
        self.is_first = false;
        Some((is_first, next))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

struct WithLast<I: Iterator> {
    iter: I,
    last: Option<I::Item>
}

impl<I: Iterator> Iterator for WithLast<I> {
    type Item = (bool, I::Item);

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter.next() {
            Some(next) => Some((false, next)),
            None => {
                let last = self.last.take()?;
                Some((true, last))
            }
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let (lo, hi) = self.iter.size_hint();
        let last = self.last.is_some() as usize;
        (lo + last, hi.map(|e| e + last))
    }
}
