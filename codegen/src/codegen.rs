use crate::Write;
use crate::element::*;

const P1: &str = "    ";
const P2: &str = "        ";
const P3: &str = "            ";
const P4: &str = "                ";

// prelude
const ENCODE_TRAIT: &str = "Encode";

const PRELUDE: &str = "
#![allow(unused_imports)]
use std::num::NonZeroU32;
use std::os::fd::RawFd;

macro_rules! roundup_4 {
    ($n:expr) => { ((($n) + 3usize) & (usize::MAX << 2)) };
}";

impl Protocol {
    pub fn generate_header(&self, o: &mut impl Write) {
        let Self {
            name,
            copyright,
            description: _,
        } = self;

        writeln!(o, "//! {name}");
        writeln!(o, "//!");
        if let Some(cp) = copyright {
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
        let Self {
            name,
            version,
            frozen,
            description: _,
        } = self;

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
        let Op {
            kind,
            name,
            destructor,
            since,
            deprecated_since,
            args,
            description: _,
        } = self;

        let is_request = kind.is_request();
        let is_event = kind.is_event();
        let lifetime = if Arg::need_lifetime(args) { "<'a>" } else { "" };
        let mod_name = match &**name {
            b"move" => "r#move",
            _ => name.as_str(),
        };
        let name = name.to_camel_case();

        writeln!(o);

        if let Some(since) = since {
            writeln!(o, "{P1}/// since: {since}");
        }
        if let Some(dep_since) = deprecated_since {
            writeln!(o, "{P1}/// deprecated-since: {dep_since}");
        }

        // ===== mod =====
        writeln!(o, "{P1}pub mod {mod_name} {{");
        writeln!(o, "{P2}use super::*;");
        writeln!(o, "{P2}pub const OPCODE: u32 = {opcode};");
        writeln!(o, "{P2}pub const IS_REQUEST: bool = {is_request};");
        writeln!(o, "{P2}pub const IS_EVENT: bool = {is_event};");
        writeln!(o, "{P2}pub const IS_DESTRUCTOR: bool = {destructor};");
        writeln!(o);

        // ===== fn encode() =====
        writeln!(o, "{P2}#[inline]");
        write!(o, "{P2}pub const fn encode{lifetime}(");
        for (is_first, arg) in args.with_first() {
            if !is_first {
                write!(o, ", ");
            }
            let ty = arg.to_rust_type();
            write!(o, "{name}: {ty}");
        }
        writeln!(o, ") -> {name}{lifetime} {{");
        write!(o, "{P3}{name} {{ ");
        for (is_first, arg) in args.with_first() {
            if !is_first {
                write!(o, ", ");
            }
            write!(o, "{}", arg.name);
        }
        writeln!(o, " }}");
        writeln!(o, "{P2}}}");
        writeln!(o);

        // ===== struct =====
        write!(o, "{P2}pub struct {name}{lifetime} {{");
        writeln!(o, "{}", &"}"[..args.is_empty() as usize]);
        for arg in args {
            const PAD: &str = P3;
            let Arg {
                name,
                ty,
                interface,
                allow_null,
                enum_name,
                summary: _,
                description: _,
            } = arg;

            write!(o, "{PAD}/// type: {}", ty.to_wl_type());
            if let Some(iface) = interface {
                write!(o, "<{iface}>");
            }
            writeln!(o);
            if let Some(enum_name) = enum_name {
                writeln!(o, "{PAD}/// enum: {enum_name}");
            }

            let ty_name = arg.to_rust_type();

            write!(o, "{PAD}pub {name}: ");
            if *allow_null {
                writeln!(o, "Option<{ty_name}>,");
            } else {
                writeln!(o, "{ty_name},");
            }
        }
        if !args.is_empty() {
            writeln!(o, "{P2}}}");
        }
        writeln!(o);

        // ===== impl Encoded =====
        writeln!(o, "{P2}impl{lifetime} {name}{lifetime} {{");

        // ===== fn size() =====
        writeln!(o, "{P3}#[inline]");
        write!(o, "{P3}pub const fn size(&self) -> usize {{");
        if args.is_empty() {
            writeln!(o, " 0 }}");
        } else {
            write!(o, "\n{P4}");
            let is_lf = args.len() > 3;
            for (is_first, arg) in args.with_first() {
                if !is_first {
                    if is_lf {
                        write!(o, "{P4}    + ");
                    } else {
                        write!(o, " + ");
                    }
                }
                // write!(o, "{ENCODE_TRAIT}::size(&self.{})", arg.name);
                arg.generate_size(o);
                if is_lf {
                    writeln!(o);
                }
            }
            if !is_lf {
                writeln!(o);
            }
            writeln!(o, "{P3}}}");
        }
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
        let Self {
            name,
            description,
            since,
            bitfield,
            entries,
        } = self;
        let _ = description;

        writeln!(o);

        // if let Some(desc) = description {
        //     desc.generate(P1, o);
        //     writeln!(o, "{P1}///");
        // }
        if let Some(since) = since {
            writeln!(o, "{P1}/// since: {since}");
        }
        writeln!(o, "{P1}/// bitfield: {bitfield}");

        let name = name.to_camel_case();

        writeln!(o, "{P1}pub enum {name} {{");
        for entry in entries {
            let Entry {
                name,
                value,
                since,
                deprecated_since,
                summary: _,
                description: _,
            } = entry;

            if let Some(since) = since {
                writeln!(o, "{P2}/// since: {since}");
            }
            if let Some(dep_since) = deprecated_since {
                writeln!(o, "{P2}/// deprecated-since: {dep_since}");
            }

            let name = name.to_camel_case();

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

    fn generate_size(&self, o: &mut impl Write) {
        let name = self.name.as_str();
        match self.ty {
            Type::Int => write!(o, "size_of::<i32>()"),
            Type::Uint | Type::Object => write!(o, "size_of::<u32>()"),
            Type::Fixed => write!(o, "size_of::<f32>()"),
            Type::String => write!(o, "(size_of::<u32>() + roundup_4!({name}.len() + 1))"),
            Type::Array => write!(o, "(size_of::<u32>() + roundup_4!({name}.len()))"),
            Type::Fd => {}
            Type::NewId => if self.interface.is_some() {
                write!(o, "size_of::<u32>()");
            } else {
                write!(o, "(size_of::<u32>() + roundup_4!({name}.len() + 1)) + ");
                write!(o, "size_of::<u64>()");
            }
        }
    }

    pub fn to_rust_type(&self) -> &'static str {
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
