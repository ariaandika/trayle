use crate::Write;
use crate::element::*;

const P1: &str = "    ";
const P2: &str = "        ";
const P3: &str = "            ";
const P4: &str = "                ";

const PRELUDE: &str = "\
#![allow(unused_imports)]
use std::num::NonZeroU32;
use std::os::fd::RawFd;
use std::ptr::copy_nonoverlapping;

// The panic code path was put into a cold function to not bloat the call site.
#[cfg_attr(not(panic = \"immediate-abort\"), inline(never), cold)]
#[cfg_attr(panic = \"immediate-abort\", inline)]
#[track_caller]
fn len_mismatch_fail(src_len: usize, dst_len: usize) -> ! {
    panic!(
        \"destination slice length ({dst_len}) does not match the required slice length ({src_len})\"
    );
}

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

struct OpContext<'a> {
    lifetime: &'static str,
    mod_name: &'a str,
    struct_name: CamelCase<'a>,
}

impl Op {
    pub fn generate(&self, opcode: u32, o: &mut impl Write) {
        let fd_count = self.args.iter().map(|e|e.is_fd() as u32).sum::<u32>();
        let require_fd = match fd_count {
            0 => false,
            1 => true,
            _ => panic!("multiple fd is not yet supported"),
        };

        if Arg::encodables(&self.args).count() == 0 {
            return self.generate_empty_encodable(opcode, o);
        }

        let mod_name = match &*self.name {
            b"move" => "r#move",
            _ => self.name.as_str(),
        };
        let struct_name = camel_case(&self.name);
        let lifetime = if Arg::need_lifetime(&self.args) { "<'a>" } else { "" };

        let cx = OpContext {
            lifetime,
            mod_name,
            struct_name,
        };

        writeln!(o);

        self.generate_constructor(&cx, o);
        self.generate_mod_header(opcode, &cx, o);
        self.generate_struct(&cx, o);

        writeln!(o, "{P2}impl{lifetime} {struct_name}{lifetime} {{");
        self.generate_fn_size(o);
        self.generate_fn_copy_to(require_fd, o);
        self.generate_fn_copy_to_unchecked(require_fd, o);
        writeln!(o, "{P2}}}");

        self.generate_mod_trailer(o);
    }

    fn generate_empty_encodable(&self, opcode: u32, o: &mut impl Write) {
        let mod_name = match &*self.name {
            b"move" => "r#move",
            _ => self.name.as_str(),
        };
        let struct_name = camel_case(&self.name);
        let full_name = format_args!("{mod_name}::{struct_name}");
        let cx = OpContext {
            lifetime: "",
            mod_name,
            struct_name,
        };

        writeln!(o);
        writeln!(
            o,
            "{P1}#[inline]\n\
            {P1}pub const fn {mod_name}() -> {full_name} {{\n\
            {P1}    {full_name} {{}}\n\
            {P1}}}\n"
        );
        self.generate_mod_header(opcode, &cx, o);
        writeln!(
            o,
            "{P2}pub struct {struct_name} {{}}\n\n\
            {P2}impl {struct_name} {{\n\
            {P2}    #[inline]\n\
            {P2}    pub const fn size(&self) -> usize {{\n\
            {P2}        0\n\
            {P2}    }}"
        );
        writeln!(o, "{P2}}}");

        self.generate_mod_trailer(o);
    }

    fn generate_constructor(&self, cx: &OpContext, o: &mut impl Write) {
        let OpContext { lifetime, mod_name, struct_name } = cx;
        let full_name = format_args!("{mod_name}::{struct_name}");
        let args = &self.args;

        // docs
        for arg in args {
            let name = &arg.name;
            let ty = arg.ty.to_wl_type();
            let n = if arg.allow_null { " | null" } else { "" };

            write!(o, "{P1}/// {name}: ");
            match &arg.enum_name {
                Some(enum_name) => write!(o, "{enum_name}"),
                None => match arg.interface.as_ref() {
                    Some(iface) => write!(o, "{ty}<{iface}>"),
                    None => write!(o, "{ty}"),
                }
            }
            writeln!(o, "{n}");
        }
        write!(o, "{P1}#[inline]\n{P1}pub const fn {mod_name}{lifetime}(");
        // arguments
        for (is_first, arg) in Arg::encodables(args).with_first() {
            let sep = if is_first { "" } else { ", " };
            let name = &arg.name;
            let ty = arg.to_rust_type();
            let ty = match arg.allow_null {
                true => format_args!("Option<{ty}>"),
                false => format_args!("{ty}")
            };
            let new_id = if arg.is_implicit_new_id() {
                format_args!("{name}_name: &'a str, {name}_version: u32, ")
            } else {
                format_args!("")
            };
            write!(o, "{sep}{new_id}{name}: {ty}");
        }
        write!(o, ") -> {full_name}{lifetime} {{\n{P2}{full_name} {{ ");
        // body
        for (is_first, arg) in Arg::encodables(args).with_first() {
            let name = &arg.name;
            let sep = if is_first { "" } else { ", " };
            let new_id = if arg.is_implicit_new_id() {
                format_args!("{name}_name, {name}_version, ")
            } else {
                format_args!("")
            };
            write!(o, "{sep}{new_id}{name}");
        }
        writeln!(o, " }}\n{P1}}}\n");
    }

    fn generate_mod_header(&self, opcode: u32, cx: &OpContext, o: &mut impl Write) {
        writeln!(o, "{P1}/// {}", self.kind.as_str());

        if let Some(since) = self.since {
            writeln!(o, "{P1}/// since: {since}");
        }
        if let Some(dep_since) = self.deprecated_since {
            writeln!(o, "{P1}/// deprecated-since: {dep_since}");
        }
        let mod_name = cx.mod_name;
        let dtor = self.destructor;
        writeln!(
            o,
            "{P1}pub mod {mod_name} {{\n\
            {P2}use super::*;\n\
            {P2}pub const OPCODE: u32 = {opcode};\n\
            {P2}pub const IS_DESTRUCTOR: bool = {dtor};\n"
        );
    }

    fn generate_mod_trailer(&self, o: &mut impl Write) {
        writeln!(o, "{P1}}}");
    }

    fn generate_struct(&self, cx: &OpContext, o: &mut impl Write) {
        let OpContext { lifetime, struct_name, .. } = cx;
        let Self { args, .. } = self;

        writeln!(o, "{P2}pub struct {struct_name}{lifetime} {{");
        for arg in Arg::encodables(args) {
            let name = &arg.name;
            let rust_ty = arg.to_rust_type();
            let wl_ty = arg.ty.to_wl_type();

            if arg.is_implicit_new_id() {
                writeln!(o,
                    "{P3}/// type: new_id.string\n\
                    {P3}pub {name}_name: &'a str,\n\
                    {P3}/// type: new_id.uint\n\
                    {P3}pub {name}_version: u32,"
                );
            }
            write!(o, "{P3}/// type: ");
            match &arg.enum_name {
                Some(enum_name) => writeln!(o, "{enum_name}"),
                None => match &arg.interface {
                    Some(iface) => writeln!(o, "{wl_ty}<{iface}>"),
                    None => writeln!(o, "{wl_ty}"),
                }
            }
            let ty = match arg.allow_null {
                true => format_args!("Option<{rust_ty}>"),
                false => format_args!("{rust_ty}")
            };
            writeln!(o, "{P3}pub {name}: {ty},");
        }
        writeln!(o, "{P2}}}\n");
    }

    fn generate_fn_size(&self, o: &mut impl Write) {
        let constant_size = self.args.iter().map(Arg::constant_size).sum::<u32>();

        writeln!(o, "{P3}#[inline]\n{P3}pub fn size(&self) -> usize {{");
        write!(o, "{P4}{constant_size}");
        for arg in &self.args {
            let name = &arg.name;
            if matches!(arg.ty, Type::String) {
                if arg.allow_null {
                    write!(o, " + self.{name}.map(|e|roundup4(e.len() + 1)).unwrap_or_default()");
                } else {
                    write!(o, " + roundup4(self.{name}.len() + 1)");
                }
            }
            if matches!(arg.ty, Type::Array) {
                write!(o, " + roundup4(self.{name}.len())");
            }
            if arg.is_implicit_new_id() {
                write!(o, " + roundup4(self.{name}_name.len() + 1)");
            }
        }
        writeln!(o, "\n{P3}}}\n");
    }

    fn generate_fn_copy_to(&self, require_fd: bool, o: &mut impl Write) {
        if require_fd {
            writeln!(o, "{P3}/// Require fd.");
        }
        writeln!(
            o,
            "{P3}/// # Panics\n\
            {P3}///\n\
            {P3}/// Panic if destination bytes length does not equal to required size.\n\
            {P3}#[inline]\n\
            {P3}pub fn copy_to(&self, buf: &mut [u8]) {{\n\
            {P3}    if self.size() != buf.len() {{\n\
            {P3}        len_mismatch_fail(self.size(), buf.len());\n\
            {P3}    }}\n\
            {P3}    // SAFETY: `buf` length is equal to required size\n\
            {P3}    unsafe {{ self.copy_to_raw(buf.as_mut_ptr()) }};\n\
            {P3}}}\n"
        );
    }

    fn generate_fn_copy_to_unchecked(&self, require_fd: bool, o: &mut impl Write) {
        const P5: &str = "                    ";
        let end_idx = Arg::encodables(&self.args).count() - 1;
        let is_mut = if end_idx == 0 {
            ""
        } else {
            "mut "
        };

        if require_fd {
            writeln!(o, "{P3}/// Require fd.");
        }
        writeln!(
            o,
            "{P3}/// # Safety\n\
            {P3}///\n\
            {P3}/// Given pointer must be valid for write until [`size()`] length.\n\
            {P3}///\n\
            {P3}/// [`size()`]: Self::size\n\
            {P3}pub unsafe fn copy_to_raw(&self, {is_mut}ptr: *mut u8) {{\n\
            {P3}    unsafe {{"
        );
        // encoding
        for (i, arg) in Arg::encodables(&self.args).enumerate() {
            let is_not_last = i != end_idx;
            let name = &arg.name;
            let adv = match arg.ty {
                Type::Int => {
                    writeln!(o, "{P5}ptr.cast::<i32>().write(self.{name});");
                    format_args!("{P5}ptr = ptr.add(4);")
                },
                Type::Uint => {
                    writeln!(o, "{P5}ptr.cast::<u32>().write(self.{name});");
                    format_args!("{P5}ptr = ptr.add(4);")
                },
                Type::Fixed => {
                    writeln!(o, "{P5}ptr.cast::<i32>().write((self.{name} * 256.0).round() as i32);");
                    format_args!("{P5}ptr = ptr.add(4);")
                },
                Type::Object => {
                    let suffix = match arg.allow_null {
                        true => ".unwrap_or_default()",
                        false => ""
                    };
                    writeln!(o, "{P5}ptr.cast::<u32>().write(self.{name}{suffix});");
                    format_args!("{P5}ptr = ptr.add(4);")
                },
                Type::String => {
                    if arg.allow_null {
                        let write = if is_not_last { "write" } else { "_" };
                        writeln!(
                            o,
                            "{P5}let {write} = match self.{name} {{\n\
                            {P5}    Some(s) => {{\n\
                            {P5}        let len = s.len() as u32;\n\
                            {P5}        ptr.cast::<u32>().write(len + 1);\n\
                            {P5}        copy_nonoverlapping(s.as_ptr(), ptr.add(4), len as usize);\n\
                            {P5}        ptr.add((4 + len) as usize).write(0);\n\
                            {P5}        4 + roundup4((len + 1) as usize)\n\
                            {P5}    }}\n\
                            {P5}    None => {{\n\
                            {P5}        ptr.cast::<u32>().write(0);\n\
                            {P5}        4\n\
                            {P5}    }}\n\
                            {P5}}};"
                        );
                        format_args!("{P5}ptr = ptr.add(write);")
                    } else {
                        writeln!(
                            o,
                            "{P5}let len = self.{name}.len() as u32;\n\
                            {P5}ptr.cast::<u32>().write(len + 1);\n\
                            {P5}copy_nonoverlapping(self.{name}.as_ptr(), ptr.add(4), len as usize);\n\
                            {P5}ptr.add((4 + len) as usize).write(0);\n"
                        );
                        format_args!("{P5}ptr = ptr.add(4 + roundup4((len + 1) as usize));")
                    }
                },
                Type::Array => {
                    writeln!(
                        o,
                        "{P5}let len = self.{name}.len() as u32;\n\
                        {P5}ptr.cast::<u32>().write(len);\n\
                        {P5}copy_nonoverlapping(self.{name}.as_ptr(), ptr.add(4), len as usize);"
                    );
                    format_args!("{P5}ptr = ptr.add(4 + roundup4(len as usize));")
                },
                Type::Fd => format_args!(""),
                Type::NewId => if arg.is_implicit_new_id() {
                    writeln!(
                        o,
                        "{P5}let len = self.{name}_name.len() as u32;\n\
                        {P5}ptr.cast::<u32>().write(len + 1);\n\
                        {P5}copy_nonoverlapping(self.{name}_name.as_ptr(), ptr.add(4), len as usize);\n\
                        {P5}ptr.add((4 + len) as usize).write(0);\n\
                        {P5}ptr = ptr.add(4 + roundup4(len as usize));\n\
                        {P5}ptr.cast::<u32>().write(self.{name}_version);\n\
                        {P5}ptr.add(4).cast::<u32>().write(self.{name});"
                    );
                    format_args!("{P5}ptr = ptr.add(8);")
                } else {
                    writeln!(o, "{P5}ptr.cast::<u32>().write(self.{name});");
                    format_args!("{P5}ptr = ptr.add(4);")
                }
            };
            if is_not_last {
                writeln!(o, "{adv}");
            }
        }
        writeln!(o, "{P3}    }}\n{P3}}}");
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
            .any(|arg| matches!(arg.ty, Type::String | Type::Array) || arg.is_implicit_new_id())
    }

    fn is_fd(&self) -> bool {
        matches!(self.ty, Type::Fd)
    }

    fn is_implicit_new_id(&self) -> bool {
        matches!(self.ty, Type::NewId) && self.interface.is_none()
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

    fn to_rust_type(&self) -> &'static str {
        match self.ty {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => "&'a str",
            Type::Array => "&'a [u8]",
            Type::Fd => "RawFd",
            Type::NewId => "u32",
            Type::Object => "u32",
        }
    }

    fn encodables(args: &[Arg]) -> impl Iterator<Item = &Arg> {
        args.iter().filter(|e| !e.is_fd())
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

#[derive(Clone, Copy)]
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
    type Iter: Iterator;

    fn with_first(self) -> WithFirst<Self::Iter>;
}

impl<I: IntoIterator> IterExt for I {
    type Iter = I::IntoIter;

    fn with_first(self) -> WithFirst<Self::Iter> {
        WithFirst {
            iter: self.into_iter(),
            is_first: true,
        }
    }

    // fn with_last(&self) -> WithLast<Self::Iter> {
    //     match self.split_last() {
    //         Some((last, rest)) => WithLast {
    //             iter: rest.iter(),
    //             last: Some(last),
    //         },
    //         None => WithLast {
    //             iter: self.iter(),
    //             last: None,
    //         },
    //     }
    // }
}

// impl<T> IterExt for Vec<T> {
//     type Iter<'a>
//         = std::slice::Iter<'a, T>
//     where
//         Self: 'a;
//
//     fn with_first<'a>(&'a self) -> WithFirst<Self::Iter<'a>> {
//         WithFirst {
//             iter: self.iter(),
//             is_first: true,
//         }
//     }
//
//     fn with_last<'a>(&'a self) -> WithLast<Self::Iter<'a>> {
//         match self.split_last() {
//             Some((last, rest)) => WithLast {
//                 iter: rest.iter(),
//                 last: Some(last),
//             },
//             None => WithLast {
//                 iter: self.iter(),
//                 last: None,
//             },
//         }
//     }
// }

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

// struct WithLast<I: Iterator> {
//     iter: I,
//     last: Option<I::Item>
// }
//
// impl<I: Iterator> Iterator for WithLast<I> {
//     type Item = (bool, I::Item);
//
//     fn next(&mut self) -> Option<Self::Item> {
//         match self.iter.next() {
//             Some(next) => Some((false, next)),
//             None => {
//                 let last = self.last.take()?;
//                 Some((true, last))
//             }
//         }
//     }
//
//     fn size_hint(&self) -> (usize, Option<usize>) {
//         let (lo, hi) = self.iter.size_hint();
//         let last = self.last.is_some() as usize;
//         (lo + last, hi.map(|e| e + last))
//     }
// }
