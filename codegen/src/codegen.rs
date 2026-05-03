use crate::Write;
use crate::element::*;

macro_rules! or_empty {
    (Some($id:ident), $($tt:tt)*) => {
        match $id {
            Some($id) => format_args!($($tt)*),
            None => format_args!(""),
        }
    };
    ($b:expr, $($tt:tt)*) => {
        if $b {
            format_args!($($tt)*)
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
#![allow(unused_imports)]
#![allow(unsafe_op_in_unsafe_fn)]
use std::num::NonZeroU32;

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

        let name_len = fmt_u32_string(name.len() as u32 + 1);
        let padding = &"\\0\\0\\0\\0"[..(pad_len * 2) as usize];
        let version = fmt_u32_string(version);
        writeln!(o, "{P1}pub static NEW_ID: [u8; {size}] = *b\"{name_len}{name}\\0{padding}{version}\";");
    }

    pub fn generate_trailer(o: &mut impl Write) {
        writeln!(o, "}}");
    }
}

struct OpContext<'a> {
    mod_name: ModName<'a>,
    fd_count: u32,
    args: Args<'a>,
}

impl Op {
    pub fn generate(&self, opcode: u16, o: &mut impl Write) {
        let args = Args(self.args.as_slice());
        let cx = OpContext {
            mod_name: ModName::new(self.name.as_str()),
            fd_count: args.fd_count(),
            args,
        };

        writeln!(o);
        self.generate_mod_header(opcode, &cx, o);
        self.generate_fn_size(&cx, o);
        self.generate_fn_encode(&cx, o);
        self.generate_mod_trailer(o);
    }

    fn generate_mod_header(&self, opcode: u16, cx: &OpContext, o: &mut impl Write) {
        let mod_name = cx.mod_name;
        let dtor = self.destructor;
        let kind = self.kind.as_str();

        let dtor_doc = or_empty!(self.destructor, ", type \"destructor\"");
        let since = or_empty!(
            self.since.is_some(),
            "{P1}/// since: {}",
            self.since.unwrap()
        );
        let dep_since = or_empty!(
            self.deprecated_since.is_some(),
            "{P1}/// deprecated-since: {}",
            self.deprecated_since.unwrap()
        );
        write!(
            o,
            "\
            {P1}/// {kind}, opcode `{opcode}`{dtor_doc}\n\
            {since}\
            {dep_since}\
            {P1}pub mod {mod_name} {{\n\
            {P2}use super::*;\n\
            {P2}pub const OPCODE: u16 = {opcode};\n\
            {P2}pub const IS_DESTRUCTOR: bool = {dtor};\n\
            "
        );
    }

    fn generate_mod_trailer(&self, o: &mut impl Write) {
        writeln!(o, "{P1}}}");
    }

    fn generate_fn_size(&self, cx: &OpContext, o: &mut impl Write) {
        let constant_size = cx.args.constant_size_sum();
        write!(o, "\n{P2}pub const fn size(");
        for (i, arg) in cx.args.dynamic_sizes().enumerate() {
            let name = &arg.name;
            let rust_ty = arg.to_rust_type();

            if i != 0 {
                write!(o, ", ");
            }
            if arg.is_implicit_new_id() {
                write!(o, "encoded_{name}_size: u16");
            } else {
                write!(o, "{name}: {rust_ty}");
            }
        }
        write!(o, ") -> u16 {{\n{P3}{constant_size}");
        for arg in cx.args.dynamic_sizes() {
            let name = &arg.name;

            write!(o, " + ");
            if arg.is_implicit_new_id() {
                write!(o, "encoded_{name}_size");
            } else if arg.allow_null {
                write!(
                    o,
                    "match {name} {{\n\
                    {P4}Some(s) => roundup4(s.len() as u16 + 1),\n\
                    {P4}None => 0,\n\
                    {P3}}}"
                )
            } else {
                write!(o, "roundup4({name}.len() as u16");
                if matches!(arg.ty, Type::String) {
                    write!(o, " + 1")
                }
                write!(o, ")");
            }
        }
        writeln!(o, "\n{P2}}}");
    }

    fn generate_fn_encode(&self, cx: &OpContext, o: &mut impl Write) {
        let encodable_len = self.args.len() as u32 - cx.fd_count;
        let fd = or_empty!(cx.fd_count != 0, "{P2}/// Require fd.\n{P2}///\n");
        let fptr = if encodable_len == 0 { "_" } else { "ptr" };
        let fmut = or_empty!(encodable_len > 1, "mut ");
        let arguments = std::fmt::from_fn(|f|{
            for arg in cx.args.encodables() {
                let name = &arg.name;
                let rust_ty = arg.to_rust_type();
                if arg.is_implicit_new_id() {
                    write!(f, "encoded_{name}: &[u8], ")?;
                }
                write!(f, "{name}: {rust_ty}")?;
                write!(f, ", ")?;
            }
            Ok(())
        });
        let body = std::fmt::from_fn(|f|{
            for (i, arg) in cx.args.encodables().enumerate() {
                let is_last = i as u32 == (encodable_len - 1);
                let name = &arg.name;
                let adv = match arg.ty {
                    Type::Int | Type::Uint | Type::Object => {
                        let ty = arg.to_rust_type();
                        writeln!(f, "{P3}ptr.cast::<{ty}>().write({name});")?;
                        format_args!("{P3}ptr = ptr.add(4);")
                    },
                    Type::Fixed => {
                        writeln!(f, "{P3}ptr.cast::<i32>().write(({name} * 256.0).round() as i32);")?;
                        format_args!("{P3}ptr = ptr.add(4);")
                    },
                    Type::Fd => format_args!(""),
                    Type::Array => {
                        writeln!(
                            f,
                            "{P3}let len = {name}.len() as u16;\n\
                            {P3}ptr.cast::<u32>().write(len as u32);\n\
                            {P3}ptr.add(4).copy_from_nonoverlapping({name}.as_ptr(), len as usize);"
                        )?;
                        format_args!("{P4}ptr = ptr.add((4 + roundup4(len)) as usize);")
                    },
                    Type::NewId => if arg.is_implicit_new_id() {
                        writeln!(
                            f,
                            "{P3}ptr.copy_from_nonoverlapping(encoded_{name}.as_ptr(), encoded_{name}.len());\n\
                            {P3}ptr.add(encoded_{name}.len()).cast::<u32>().write({name});"
                        )?;
                        format_args!("{P3}ptr = ptr.add(encoded_{name}.len() + 4);")
                    } else {
                        writeln!(f, "{P3}ptr.cast::<u32>().write({name});")?;
                        format_args!("{P3}ptr = ptr.add(4);")
                    }
                    Type::String => {
                        if arg.allow_null {
                            let write = if is_last { "_" } else { "write" };
                            let some_len = or_empty!(!is_last, "{P4}    4 + roundup4(len + 1)\n");
                            let none_len = or_empty!(!is_last, "{P4}    4\n");
                            writeln!(
                                f,
                                "{P3}let {write} = match {name} {{\n\
                                {P3}    Some(s) => {{\n\
                                {P3}        let len = s.len() as u16;\n\
                                {P3}        ptr.cast::<u32>().write((len + 1) as u32);\n\
                                {P3}        ptr.add(4).copy_from_nonoverlapping(s.as_ptr(), len as usize);\n\
                                {P3}        ptr.add((4 + len) as usize).write(0);\n\
                                {some_len}\
                                {P3}    }}\n\
                                {P3}    None => {{\n\
                                {P3}        ptr.cast::<u32>().write(0);\n\
                                {none_len}\
                                {P3}    }}\n\
                                {P3}}};"
                            )?;
                            format_args!("{P4}ptr = ptr.add(write as usize);")
                        } else {
                            writeln!(
                                f,
                                "{P3}let len = {name}.len() as u16;\n\
                                {P3}ptr.cast::<u32>().write((len + 1) as u32);\n\
                                {P3}ptr.add(4).copy_from_nonoverlapping({name}.as_ptr(), len as usize);\n\
                                {P3}ptr.add((4 + len) as usize).write(0);"
                            )?;
                            format_args!("{P3}ptr = ptr.add((4 + roundup4(len + 1)) as usize);")
                        }
                    },
                };
                if !is_last {
                    writeln!(f, "{adv}")?;
                }
            }

            Ok(())
        });

        write!(
            o,
            "\n\
            {fd}\
            {P2}/// # Safety\n\
            {P2}///\n\
            {P2}/// Given pointer must be valid for write until required length.\n\
            {P2}pub unsafe fn encode({arguments}{fmut}{fptr}: *mut u8) {{\n\
            {body}\
            {P2}}}\n\
            "
        );
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

struct Args<'a>(&'a [Arg]);

impl Args<'_> {
    fn fd_count(&self) -> u32 {
        self.0.iter().map(|e| e.is_fd() as u32).sum()
    }

    /// non fd arguments
    fn encodables(&self) -> impl Iterator<Item = &Arg> {
        self.0.iter().filter(|e| !e.is_fd())
    }

    /// string or array or implicit new_id
    fn dynamic_sizes(&self) -> impl Iterator<Item = &Arg> {
        self.0.iter().filter(|e| e.is_dynamic_size() || e.is_implicit_new_id())
    }

    fn constant_size_sum(&self) -> u16 {
        self.0
            .iter()
            .map(|e| match e.ty {
                Type::Int
                | Type::Uint
                | Type::Fixed
                | Type::String
                | Type::Array
                | Type::Object => 4,
                Type::Fd => 0,
                Type::NewId => {
                    if e.interface.is_some() {
                        4
                    } else {
                        12
                    }
                }
            })
            .sum::<u16>()
    }
}

impl<'a> IntoIterator for &Args<'a> {
    type Item = &'a Arg;

    type IntoIter = std::slice::Iter<'a, Arg>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

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

    fn to_rust_type(&self) -> &'static str {
        match self.ty {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => if self.allow_null {
                "Option<&str>"
            } else {
                "&str"
            },
            Type::Array => "&[u8]",
            Type::Fd => "RawFd",
            Type::NewId => "u32",
            Type::Object => if self.allow_null {
                "u32"
            } else {
                "NonZeroU32"
            },
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
    pub fn as_str(&self) -> &'static str {
        match self {
            OpKind::Request => "request",
            OpKind::Event => "event",
        }
    }
}

// ===== util =====

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
