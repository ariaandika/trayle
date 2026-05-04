use std::fmt::Display;

use crate::Write;
use crate::element::*;

macro_rules! deref {
    ($me:ident$(<$lf:lifetime>)?,$t:ident$(<$lf2:lifetime>)?,$id:tt) => {
        impl$(<$lf>)? std::ops::Deref for $me $(<$lf>)? {
            type Target = $t $(<$lf2>)?;

            fn deref(&self) -> &Self::Target {
                &self.$id
            }
        }
    };
    ($me:ty,$t:ty,$id:tt) => {
        impl std::ops::Deref for $me {
            type Target = $t;

            fn deref(&self) -> &Self::Target {
                &self.$id
            }
        }
    };
}


macro_rules! or_empty {
    ($b:ident, $($tt:tt)*) => {
        if $b {
            format_args!($($tt)*)
        } else {
            format_args!("")
        }
    };
    (let $b:ident, $($tt:tt)*) => {
        if $b.is_some() {
            format_args!($($tt)*,$b.unwrap())
        } else {
            format_args!("")
        }
    };
    ($e:expr, $($tf:tt)*) => {
        if $e {
            format_args!($($tf)*)
        } else {
            format_args!("")
        }
    };
}

const P1: &str = "    ";
const P2: &str = "        ";
const P3: &str = "            ";
const P4: &str = "                ";

const HEADERS: &str = "\
#![allow(unsafe_op_in_unsafe_fn)]

const fn roundup4(value: u16) -> u16 {
    (value + 3) & (u16::MAX << 2)
}";

impl Protocol {
    pub fn generate_header(&self, o: &mut impl Write) {
        let name = &self.name;
        let cp = self.copyright.as_ref();

        writeln!(o, "//! {name}");
        if let Some(cp) = cp {
            writeln!(o, "//!");
            writeln!(o, "//! ===== COPYRIGHT =====");
            writeln!(o, "//!");
            for line in cp.as_str().lines().map(str::trim_start) {
                let sp = &" "[line.is_empty() as usize..];
                writeln!(o, "//!{sp}{line}");
            }
            writeln!(o, "//!");
            writeln!(o, "//! ===== COPYRIGHT =====");
        }
        writeln!(o, "{HEADERS}");
    }
}

impl Interface {
    pub fn generate_header(&self, o: &mut impl Write) {
        let name = &self.name;
        let version = self.version;
        let frozen = self.frozen;
        write!(
            o,
            "\n\
            pub mod {name} {{\n\
            {P1}use super::*;\n\
            {P1}pub const VERSION: u32 = {version};\n\
            {P1}pub const FROZEN: bool = {frozen};\n\
            "
        );
        let name_len = name.len() as u32 + 1;
        let pad_len = roundup4(name_len) - name_len;
        let size = 4 + roundup4(name_len) + 4;

        let encoded_name_len = fmt_u32_string(name.len() as u32 + 1);
        let padding = &"\\0\\0\\0\\0"[..(pad_len * 2) as usize];
        let version = fmt_u32_string(version);
        write!(
            o,
            "\
            {P1}pub const NAME_LEN: u16 = {name_len};\n\
            {P1}pub static NEW_ID: [u8; {size}] = *b\"{encoded_name_len}{name}\\0{padding}{version}\";\n\
            "
        );
    }

    pub fn generate_trailer(o: &mut impl Write) {
        writeln!(o, "}}");
    }
}

impl Enum {
    pub fn generate(&self, o: &mut impl Write) {
        let name = CamelCase(self.name.as_str());
        let bitfield = self.bitfield;
        let entries = &self.entries;

        writeln!(o);
        if let Some(since) = self.since {
            writeln!(o, "{P1}/// since: {since}");
        }
        writeln!(o, "{P1}/// bitfield: {bitfield}");
        writeln!(o, "{P1}pub enum {name} {{");
        for entry in entries {
            let name = CamelCase(entry.name.as_str());
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

impl Op {
    pub fn generate(&self, opcode: u16, o: &mut impl Write) {
        // separate kind of message
        //
        // zero length message:
        // - zero size
        // - noop encoding
        // - noop decoding
        // - no lifetime
        //
        // static length message:
        // - constant size
        // - compiletime encoding checks
        // - infallible decoding
        // - no lifetime
        //
        // dynamic length message:
        // - arbitrary size
        // - runtime encoding checks, or unsafe unchecked
        // - fallible decoding
        // - requires lifetime
        //
        // with exception, message with implicit new_id:
        // - count as static length message
        // - count as dynamic length message
        let mut fd_count = 0;
        let mut const_size = 0;
        let mut dynamic_count = 0;
        let mut encodable_count = 0;
        for arg in self.encodables() {
            fd_count += arg.is_fd() as u32;
            const_size += arg.const_size();
            dynamic_count += (arg.is_dynamic_size() || arg.is_implicit_new_id()) as u32;
            encodable_count += 1;
        }

        let mod_name = ModName::new(&self.name);
        let struct_name = CamelCase(&self.name);
        let is_zero_size = encodable_count == 0;
        let is_fd = fd_count > 0;
        let is_dynamic = dynamic_count > 0;
        let dtor = self.destructor;
        let kind = self.kind.as_str();

        let lf = or_empty!(is_dynamic, "<'a>");
        let dtor_doc = or_empty!(dtor, ", type \"destructor\"");
        let fd_doc = or_empty!(is_fd, "{P2}/// Require fd.\n{P2}///\n");
        let const_size_fmt = or_empty!(!is_dynamic, "{P2}pub const SIZE: u16 = {const_size};\n");
        let unsafe_fn = or_empty!(!is_zero_size, "unsafe ");
        let safety_doc = or_empty!(
            !is_zero_size,
            "{P2}/// # Safety\n\
            {P2}///\n\
            {P2}/// Given pointer must be valid for write until required length.\n"
        );
        let fields = self.fmt_encodables(|_, arg, f| {
            let name = &arg.name;
            let rust_ty = arg.to_rust_type(true);
            if arg.is_implicit_new_id() {
                writeln!(f, "{P3}pub {name}_name: &'a [u8],\n{P3}pub {name}_version: u32,")?;
            }
            writeln!(f, "{P3}pub {name}: {rust_ty},")
        });
        let size_args = self.fmt_dynamic_sizes(|i, arg, f|{
            let name = &arg.name;
            let new_id = arg.is_implicit_new_id();
            let sep = or_empty!(i != 0, ", ");
            let suffix = or_empty!(new_id, "_name_len");
            let rust_ty = if new_id { "u16" } else { arg.to_rust_type(false) };
            write!(f, "{sep}{name}{suffix}: {rust_ty}")
        });
        let dyn_sizes = self.fmt_dynamic_sizes(|_, arg, f|{
            let name = &arg.name;
            match arg.ty {
                Type::Array => write!(f, " + {name}.len() as u16"),
                Type::String => if arg.allow_null {
                    write!(f, " + {name}.map(|s|s.len() as u16 + 1).unwrap_or(0)")
                } else {
                    write!(f, " + {name}.len() as u16 + 1")
                },
                _ => if arg.is_implicit_new_id() {
                    write!(f, " + {name}_name_len")
                } else {
                    Ok(())
                },
            }
        });
        let encode_ptr_id = match encodable_count {
            0 => format_args!("_"),
            1 => format_args!("ptr"),
            _ => format_args!("mut ptr"),
        };
        let encode_args = self.fmt_encodables(|_, arg, f|{
            let name = &arg.name;
            let rust_ty = arg.to_rust_type(false);
            if arg.is_implicit_new_id() {
                write!(f, "encoded_{name}: &[u8], ")?;
            }
            write!(f, "{name}: {rust_ty}, ")
        });
        let encode_body = self.fmt_encodables(|i, arg, f| {
            ArgEncode {
                arg,
                is_last: i as u32 == encodable_count - 1,
                p: P3,
            }
            .fmt(f)
        });

        write!(
            o,
            "\n\
            {P1}/// {kind}, opcode `{opcode}`{dtor_doc}\n\
            {P1}pub mod {mod_name} {{\n\
            {P1}    use super::*;\n\
            {P1}    pub const OPCODE: u16 = {opcode};\n\
            {P1}    pub const IS_DESTRUCTOR: bool = {dtor};\n\
            {const_size_fmt}\n\
            {P1}    pub struct {struct_name}{lf} {{\n\
            {fields}\
            {P1}    }}\n\n\
            {P1}    pub fn size({size_args}) -> u16 {{\n\
            {P1}        {const_size}{dyn_sizes}\n\
            {P1}    }}\n\n\
            {fd_doc}\
            {safety_doc}\
            {P1}    pub {unsafe_fn}fn encode({encode_args}{encode_ptr_id}: *mut u8) {{\n\
            {encode_body}\
            {P1}    }}\n\
            {P1}}}\n\
            "
        );
    }
}

deref!(Op, Vec<Arg>, args);

struct ArgEncode<'a> {
    arg: &'a Arg,
    is_last: bool,
    p: &'static str,
}

deref!(ArgEncode<'a>, Arg, arg);

impl std::fmt::Display for ArgEncode<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let p = self.p;
        let is_last = self.is_last;
        let name = &self.name;
        let adv = match self.ty {
            Type::Int | Type::Uint | Type::Object => {
                let ty = self.to_rust_type(false);
                writeln!(f, "{p}ptr.cast::<{ty}>().write({name});")?;
                format_args!("{p}ptr = ptr.add(4);")
            },
            Type::Fixed => {
                writeln!(f, "{p}ptr.cast::<i32>().write(({name} * 256.0).round() as i32);")?;
                format_args!("{p}ptr = ptr.add(4);")
            },
            Type::Fd => format_args!(""),
            Type::Array => {
                writeln!(
                    f,
                    "{p}let len = {name}.len() as u16;\n\
                    {p}ptr.cast::<u32>().write(len as u32);\n\
                    {p}ptr.add(4).copy_from_nonoverlapping({name}.as_ptr(), len as usize);"
                )?;
                format_args!("{P4}ptr = ptr.add((4 + roundup4(len)) as usize);")
            },
            Type::NewId => if self.is_implicit_new_id() {
                writeln!(
                    f,
                    "{p}ptr.copy_from_nonoverlapping(encoded_{name}.as_ptr(), encoded_{name}.len());\n\
                    {p}ptr.add(encoded_{name}.len()).cast::<u32>().write({name});"
                )?;
                format_args!("{p}ptr = ptr.add(encoded_{name}.len() + 4);")
            } else {
                writeln!(f, "{p}ptr.cast::<u32>().write({name});")?;
                format_args!("{p}ptr = ptr.add(4);")
            }
            Type::String => {
                if self.allow_null {
                    let write = if is_last { "_" } else { "write" };
                    let some_len = or_empty!(!is_last, "{P4}    4 + roundup4(len + 1)\n");
                    let none_len = or_empty!(!is_last, "{P4}    4\n");
                    writeln!(
                        f,
                        "{p}let {write} = match {name} {{\n\
                        {p}    Some(s) => {{\n\
                        {p}        let len = s.len() as u16;\n\
                        {p}        ptr.cast::<u32>().write((len + 1) as u32);\n\
                        {p}        ptr.add(4).copy_from_nonoverlapping(s.as_ptr(), len as usize);\n\
                        {p}        ptr.add((4 + len) as usize).write(0);\n\
                        {some_len}\
                        {p}    }}\n\
                        {p}    None => {{\n\
                        {p}        ptr.cast::<u32>().write(0);\n\
                        {none_len}\
                        {p}    }}\n\
                        {p}}};"
                    )?;
                    format_args!("{P4}ptr = ptr.add(write as usize);")
                } else {
                    writeln!(
                        f,
                        "{p}let len = {name}.len() as u16;\n\
                        {p}ptr.cast::<u32>().write((len + 1) as u32);\n\
                        {p}ptr.add(4).copy_from_nonoverlapping({name}.as_ptr(), len as usize);\n\
                        {p}ptr.add((4 + len) as usize).write(0);"
                    )?;
                    format_args!("{p}ptr = ptr.add((4 + roundup4(len + 1)) as usize);")
                }
            },
        };
        if !is_last {
            writeln!(f, "{adv}")?;
        }
        Ok(())
    }
}

// ===== subtypes =====

impl Arg {
    fn is_fd(&self) -> bool {
        matches!(self.ty, Type::Fd)
    }

    fn is_implicit_new_id(&self) -> bool {
        matches!(self.ty, Type::NewId) && self.interface.is_none()
    }

    /// string or array
    fn is_dynamic_size(&self) -> bool {
        matches!(self.ty, Type::String | Type::Array)
    }

    fn const_size(&self) -> u32 {
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

    fn to_rust_type(&self, is_lifetime: bool) -> &'static str {
        match self.ty {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => {
                if self.allow_null {
                    if is_lifetime {
                        "Option<&'a str>"
                    } else {
                        "Option<&str>"
                    }
                } else {
                    if is_lifetime {
                        "&'a str"
                    } else {
                        "&str"
                    }
                }
            }
            Type::Array => {
                if is_lifetime {
                    "&'a [u8]"
                } else {
                    "&[u8]"
                }
            }
            Type::Fd => "RawFd",
            Type::NewId => "u32",
            Type::Object => "u32",
        }
    }
}

impl Type {
    pub fn from_wl_type(ty: &str) -> Self {
        match ty {
            "int" => Self::Int,
            "uint" => Self::Uint,
            "fixed" => Self::Fixed,
            "string" => Self::String,
            "array" => Self::Array,
            "fd" => Self::Fd,
            "new_id" => Self::NewId,
            "object" => Self::Object,
            _ => panic!("unknown type: {ty}"),
        }
    }
}

impl OpKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            OpKind::Request => "request",
            OpKind::Event => "event",
        }
    }
}

// ===== util =====

trait ArgExt {
    fn encodables(&self) -> impl Iterator<Item = &Arg>;

    /// array, string, or implicit new id
    fn dynamic_sizes(&self) -> impl Iterator<Item = &Arg>;

    fn fmt_encodables<F>(&self, f: F) -> impl std::fmt::Display
    where
        F: Fn(usize, &Arg, &mut std::fmt::Formatter) -> std::fmt::Result,
    {
        std::fmt::from_fn(move |fmt| {
            for (i, arg) in self.encodables().enumerate() {
                f(i, arg, fmt)?;
            }
            Ok(())
        })
    }

    fn fmt_dynamic_sizes<F>(&self, f: F) -> impl std::fmt::Display
    where
        F: Fn(usize, &Arg, &mut std::fmt::Formatter) -> std::fmt::Result,
    {
        std::fmt::from_fn(move |fmt| {
            for (i, arg) in self.dynamic_sizes().enumerate() {
                f(i, arg, fmt)?;
            }
            Ok(())
        })
    }
}

impl ArgExt for Vec<Arg> {
    fn encodables(&self) -> impl Iterator<Item = &Arg> {
        self.iter().filter(|e| !e.is_fd())
    }

    fn dynamic_sizes(&self) -> impl Iterator<Item = &Arg> {
        self.iter().filter(|e| e.is_dynamic_size() || e.is_implicit_new_id())
    }
}

fn fmt_u32_string(value: u32) -> impl std::fmt::Display {
    std::fmt::from_fn(move |f| {
        for b in value.to_ne_bytes() {
            write!(f, "\\x{b:0>2x}")?;
        }
        Ok(())
    })
}

fn roundup4(value: u32) -> u32 {
    (value + 3) & (u32::MAX << 2)
}

#[derive(Clone, Copy)]
struct ModName<'a> {
    prefix: &'static str,
    name: &'a str,
}

impl<'a> ModName<'a> {
    fn new(name: &'a str) -> Self {
        // some name is a rust keyword.
        let prefix = if matches!(name, "move") { "r#" } else { "" };
        Self { prefix, name }
    }
}

impl<'a> std::fmt::Display for ModName<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.prefix, self.name)
    }
}

#[derive(Clone, Copy)]
struct CamelCase<'a>(&'a str);

impl<'a> std::fmt::Display for CamelCase<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut chars = self.0.chars();
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
