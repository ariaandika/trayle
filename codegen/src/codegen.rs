use crate::Write;
use crate::element::*;

macro_rules! or_empty {
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
use std::os::fd::RawFd;
use std::ptr::{NonNull, copy_nonoverlapping};

";

const TRAILER: &str = "
const fn roundup4(value: u16) -> u16 {
    (value + 3) & (u16::MAX << 2)
}

macro_rules! non_null {
    ($e:expr) => {
        unsafe { NonNull::new_unchecked($e.as_ptr().cast_mut())}
    }
}

use non_null;

// The panic code path was put into a cold function to not bloat the call site.
#[cfg_attr(not(panic = \"immediate-abort\"), inline(never), cold)]
#[cfg_attr(panic = \"immediate-abort\", inline)]
#[track_caller]
fn len_mismatch_fail(src_len: usize, dst_len: usize) -> ! {
    panic!(
        \"destination slice length ({dst_len}) does not match the required slice length ({src_len})\"
    );
}

#[cfg_attr(not(panic = \"immediate-abort\"), inline(never), cold)]
#[cfg_attr(panic = \"immediate-abort\", inline)]
#[track_caller]
fn excessive_length_fail(len: usize) -> ! {
    panic!(\"excessive length ({len}), cannot exceed `u16::MAX`\");
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

    pub fn generate_trailer(o: &mut impl Write) {
        writeln!(o, "{TRAILER}");
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
    struct_name: StructName<'a>,
    lifetime: &'static str,
    fd_count: u32,
    args: Args<'a>,
}

impl Op {
    pub fn generate(&self, opcode: u16, o: &mut impl Write) {
        let mod_name = ModName::new(self.name.as_str());
        let struct_name = StructName(self.name.as_str());
        let args = Args(self.args.as_slice());
        let (fd_count, dynamic) = args.data();
        let is_lifetime = dynamic > 0;
        let lifetime = if is_lifetime { "<'a>" } else { "" };
        let require_fd = match fd_count {
            0 => false,
            1 => true,
            _ => panic!("multiple fd is not yet supported"),
        };

        let cx = OpContext {
            mod_name,
            struct_name,
            lifetime,
            fd_count,
            args,
        };

        // if (self.args.len() - fd_count as usize) == 0 {
        //     return self.generate_empty_encodable(opcode, cx, o);
        // }

        writeln!(o);
        self.generate_mod_header(opcode, &cx, o);

        // - const fn size(dynamic_args) -> u16;
        // - const fn encode(args.., oid, ptr) -> u16;

        self.generate_fn_size(&cx, o);
        self.generate_fn_encode(&cx, o);

        // - const fn new() -> Sync;
        // - struct Sync;
        // - impl Sync;
        // - fn size(&self) -> u16;
        // - fn encode(&self) -> u16;

        // self.generate_constructor(&cx, o);
        // self.generate_struct(&cx, o);
        //
        // writeln!(o, "{P2}impl{lifetime} {}{lifetime} {{", cx.struct_name);
        // self.generate_size(&cx, o);
        // self.generate_encode(require_fd, o);
        // self.generate_encode_raw(require_fd, &cx, o);
        // writeln!(o, "{P2}}}");

        self.generate_mod_trailer(o);
    }

    fn generate_mod_header(&self, opcode: u16, cx: &OpContext, o: &mut impl Write) {
        let mod_name = cx.mod_name;
        let dtor = self.destructor;
        let kind = self.kind.as_str();
        let dtor_doc = if self.destructor { ", type \"destructor\"" } else { "" };

            // {P1}pub use {mod_name}::new as {mod_name};\n\n\
        writeln!(o, "{P1}/// {kind}, opcode `{opcode}`{dtor_doc}");
        if let Some(since) = self.since {
            writeln!(o, "{P1}/// since: {since}");
        }
        if let Some(dep_since) = self.deprecated_since {
            writeln!(o, "{P1}/// deprecated-since: {dep_since}");
        }
        write!(
            o,
            "\
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

    fn generate_constructor(&self, cx: &OpContext, o: &mut impl Write) {
        let OpContext { struct_name, lifetime, .. } = cx;
        let is_lifetime = !lifetime.is_empty();

        // docs
        for arg in &cx.args {
            let name = &arg.name;
            let wl_ty = WlTypeDocs(arg);
            writeln!(o, "{P2}/// {name}: {wl_ty}");
        }
        write!(
            o,
            "\
            {P2}#[doc(hidden)]\n\
            {P2}#[inline]\n\
            {P2}pub const fn new{lifetime}("
        );
        // arguments
        for (is_first, arg) in cx.args.encodables().with_first() {
            if !is_first {
                write!(o, ", ");
            }

            let name = &arg.name;
            let rust_ty = arg.to_rust_type();
            if arg.is_implicit_new_id() {
                write!(o, "{name}_name: &'a str, {name}_version: u32, ");
            }
            write!(o, "{name}: {rust_ty}");
        }
        writeln!(o, ") -> {struct_name}{lifetime} {{");
        // string extraction
        for Arg { name, allow_null, .. } in cx.args.encodables().filter(|e|e.is_dynamic_size()) {
            if *allow_null {
                writeln!(
                    o,
                    "{P3}let ({name}_ptr, {name}_len) = match {name} {{\n\
                    {P3}    Some(s) => (s.as_ptr(), s.len() as u16),\n\
                    {P3}    None => (std::ptr::null(), 0),\n\
                    {P3}}};"
                );
            } else {
                writeln!(o, "{P3}let ({name}_ptr, {name}_len) = (non_null!({name}), {name}.len() as u16);");
            }
        }
        // body
        write!(o, "{P3}{struct_name} {{ ");
        for arg in cx.args.encodables() {
            let name = &arg.name;
            match arg.kind() {
                ArgKind::Regular => write!(o, "{name}, "),
                ArgKind::ImplNewId => write!(
                    o, "{name}_name_ptr: non_null!({name}_name), {name}_name_len: {name}_name.len() as _, {name}_version, {name}, "
                ),
                ArgKind::Dynamic => write!(o, "{name}_ptr, {name}_len, "),
            }
        }
        if is_lifetime {
            write!(o, "_p: std::marker::PhantomData ");
        }
        writeln!(o, "}}\n{P2}}}\n");
    }

    fn generate_struct(&self, cx: &OpContext, o: &mut impl Write) {
        let OpContext { lifetime, struct_name, .. } = cx;
        let is_lifetime = !lifetime.is_empty();

        writeln!(o, "{P2}pub struct {struct_name}{lifetime} {{");
        for arg in cx.args.encodables() {
            let name = &arg.name;
            let rust_ty = arg.to_rust_type();
            match arg.kind() {
                ArgKind::Regular => {
                    writeln!(o, "{P3}{name}: {rust_ty},");
                }
                ArgKind::ImplNewId => {
                    writeln!(
                        o,
                        "{P3}{name}_name_ptr: NonNull<u8>,\n\
                        {P3}{name}_name_len: u16,\n\
                        {P3}{name}_version: u32,\n\
                        {P3}{name}: u32,"
                    );
                }
                ArgKind::Dynamic => {
                    write!(o, "{P3}pub(super) {name}_ptr: ");
                    if arg.allow_null {
                        writeln!(o, "*const u8,");
                    } else {
                        writeln!(o, "NonNull<u8>,");
                    }
                    writeln!(o, "{P3}pub(super) {name}_len: u16,");
                }
            }
        }
        if is_lifetime {
            writeln!(o, "{P3}pub(super) _p: std::marker::PhantomData<&'a ()>,");
        }
        writeln!(o, "{P2}}}\n");
    }

    fn generate_size(&self, cx: &OpContext, o: &mut impl Write) {
        let constant_size = 8 + self.args.iter().map(Arg::constant_size).sum::<u32>();

        writeln!(o, "{P3}#[inline]\n{P3}pub const fn size(&self) -> u16 {{");
        write!(o, "{P4}{constant_size}");
        for arg in cx.args.dynamic_sizes() {
            let name = &arg.name;
            match arg.kind() {
                ArgKind::Regular => {}
                ArgKind::ImplNewId => {
                    write!(o, " + roundup4(self.{name}_name_len + 1)");
                }
                ArgKind::Dynamic => {
                    if arg.allow_null {
                        write!(
                            o,
                            " + match NonNull::new(self.{name}_ptr)\n\
                                Some(_) => roundup4(self.{name}_len + 1),\n\
                                None => 0,\n\
                            }}"
                        );
                    } else {
                        write!(o, " + roundup4(self.{name}_len + 1)");
                    }
                }
            }
        }
        writeln!(o, "\n{P3}}}\n");
    }

    fn generate_encode(&self, require_fd: bool, o: &mut impl Write) {
        if require_fd {
            writeln!(o, "{P3}/// Require fd.\n{P3}///");
        }
        writeln!(
            o,
            "\
            {P3}/// # Panics\n\
            {P3}///\n\
            {P3}/// Panic if destination bytes length does not equal to required size.\n\
            {P3}#[inline]\n\
            {P3}pub fn encode(&self, object_id: u32, buf: &mut [u8]) {{\n\
            {P3}    if self.size() as usize != buf.len() {{\n\
            {P3}        len_mismatch_fail(self.size() as usize, buf.len());\n\
            {P3}    }}\n\
            {P3}    // SAFETY: `buf` length is equal to required size\n\
            {P3}    unsafe {{ self.encode_raw(object_id, NonNull::new_unchecked(buf.as_mut_ptr())) }};\n\
            {P3}}}\n"
        );
    }

    fn generate_encode_raw(&self, require_fd: bool, cx: &OpContext, o: &mut impl Write) {
        const P5: &str = "                    ";
        let last_idx = self.args.len() - cx.fd_count as usize - 1;

        if require_fd {
            writeln!(o, "{P3}/// Require fd.\n{P3}///");
        }
        write!(
            o,
            "\
            {P3}/// # Safety\n\
            {P3}///\n\
            {P3}/// Given pointer must be valid for write until [`size()`] length.\n\
            {P3}///\n\
            {P3}/// [`size()`]: Self::size\n\
            {P3}pub unsafe fn encode_raw(&self, object_id: u32, mut ptr: NonNull<u8>) {{\n\
            {P3}    let size = self.size();\n\
            {P3}    ptr.cast::<u32>().write(object_id);\n\
            {P3}    ptr = ptr.add(4);\n\
            "
        );
        // encoding
        for (i, arg) in cx.args.encodables().enumerate() {
            let is_not_last = i != last_idx;
            let name = &arg.name;
            let adv = match arg.ty {
                Type::Int => {
                    writeln!(o, "{P4}ptr.cast::<i32>().write(self.{name});");
                    format_args!("{P4}ptr = ptr.add(4);")
                },
                Type::Uint => {
                    writeln!(o, "{P4}ptr.cast::<u32>().write(self.{name});");
                    format_args!("{P4}ptr = ptr.add(4);")
                },
                Type::Fixed => {
                    writeln!(o, "{P4}ptr.cast::<i32>().write((self.{name} * 256.0).round() as i32);");
                    format_args!("{P4}ptr = ptr.add(4);")
                },
                Type::Object => {
                    let suffix = if arg.allow_null { "" } else { ".get()" };
                    writeln!(o, "{P4}ptr.cast::<u32>().write(self.{name}{suffix});");
                    format_args!("{P4}ptr = ptr.add(4);")
                },
                Type::String => {
                    if arg.allow_null {
                        let write: &'static str = if is_not_last { "write" } else { "_" };
                        writeln!(
                            o,
                            "{P4}let {write} = match NonNull::new(self.{name}_ptr) {{\n\
                            {P4}    Some(s_ptr) => {{\n\
                            {P4}        let len = self.{name}_len;\n\
                            {P4}        ptr.cast::<u32>().write((len + 1) as u32);\n\
                            {P4}        ptr.add(4).copy_from_nonoverlapping(s_ptr, len as usize);\n\
                            {P4}        ptr.add((4 + len) as usize).write(0);\n\
                            {P4}        4 + roundup4(len + 1)\n\
                            {P4}    }}\n\
                            {P4}    None => {{\n\
                            {P4}        ptr.cast::<u32>().write(0);\n\
                            {P4}        4\n\
                            {P4}    }}\n\
                            {P4}}};"
                        );
                        format_args!("{P4}ptr = ptr.add(write as usize);")
                    } else {
                        writeln!(
                            o,
                            "{P4}let len = self.{name}_len;\n\
                            {P4}ptr.cast::<u32>().write((len + 1) as u32);\n\
                            {P4}ptr.add(4).copy_from_nonoverlapping(self.{name}_ptr, len as usize);\n\
                            {P4}ptr.add((4 + len) as usize).write(0);\n"
                        );
                        format_args!("{P5}ptr = ptr.add(4 + roundup4(len + 1) as usize);")
                    }
                },
                Type::Array => {
                    writeln!(
                        o,
                        "{P4}let len = self.{name}_len;\n\
                        {P4}ptr.cast::<u32>().write(len as u32);\n\
                        {P4}ptr.add(4).copy_from_nonoverlapping(self.{name}_ptr, len as usize);"
                    );
                    format_args!("{P5}ptr = ptr.add(4 + roundup4(len as usize));")
                },
                Type::Fd => format_args!(""),
                Type::NewId => if arg.is_implicit_new_id() {
                    writeln!(
                        o,
                        "{P4}let len = self.{name}_name_len;\n\
                        {P4}ptr.cast::<u32>().write((len + 1) as u32);\n\
                        {P4}ptr.add(4).copy_from_nonoverlapping(self.{name}_name_ptr, len as usize);\n\
                        {P4}ptr.add((4 + len) as usize).write(0);\n\
                        {P4}ptr = ptr.add((4 + roundup4(len + 1)) as usize);\n\
                        {P4}ptr.cast::<u32>().write(self.{name}_version);\n\
                        {P4}ptr.add(4).cast::<u32>().write(self.{name});"
                    );
                    format_args!("{P4}ptr = ptr.add(12 + roundup4(len + 1) as usize);")
                } else {
                    writeln!(o, "{P4}ptr.cast::<u32>().write(self.{name});");
                    format_args!("{P4}ptr = ptr.add(4);")
                }
            };
            if is_not_last {
                writeln!(o, "{adv}");
            }
        }
        writeln!(o, "{P3}}}");
    }

    fn generate_fn_size(&self, cx: &OpContext, o: &mut impl Write) {
        let constant_size = cx.args.constant_size_sum();
        write!(o, "\n{P2}pub const fn size(");
        for (i, arg) in cx.args.dynamic_sizes().enumerate() {
            let name = &arg.name;
            let rust_ty = arg.to_rust_type_no_lifetime();

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
                write!(o, "match {name} {{\n{P4}Some(s) => roundup4(s.len() as u16 + 1),\n{P4}None => 0,\n{P3}}}")
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
        let fd = or_empty!(cx.fd_count != 0, "{P2}/// Require fd.\n{P2}///);");
        let fmut = or_empty!(encodable_len != 0, "mut ");
        let arguments = std::fmt::from_fn(|f|{
            for arg in cx.args.encodables() {
                let name = &arg.name;
                let rust_ty = arg.to_rust_type_no_lifetime();
                if arg.is_implicit_new_id() {
                    write!(f, "encoded_{name}: &[u8]")?;
                } else {
                    write!(f, "{name}: {rust_ty}")?;
                }
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
                        let ty = arg.to_rust_type_no_lifetime();
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
                        format_args!("{P4}ptr = ptr.add(4 + roundup4(len as usize));")
                    },
                    Type::NewId => if arg.is_implicit_new_id() {
                        writeln!(
                            f,
                            "{P4}let len = self.{name}_name_len;\n\
                            {P4}ptr.cast::<u32>().write((len + 1) as u32);\n\
                            {P4}ptr.add(4).copy_from_nonoverlapping(self.{name}_name_ptr, len as usize);\n\
                            {P4}ptr.add((4 + len) as usize).write(0);\n\
                            {P4}ptr = ptr.add((4 + roundup4(len + 1)) as usize);\n\
                            {P4}ptr.cast::<u32>().write(self.{name}_version);\n\
                            {P4}ptr.add(4).cast::<u32>().write(self.{name});"
                        )?;
                        format_args!("{P4}ptr = ptr.add(12 + roundup4(len + 1) as usize);")
                    } else {
                        writeln!(f, "{P4}ptr.cast::<u32>().write(self.{name});")?;
                        format_args!("{P4}ptr = ptr.add(4);")
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
                                {P3}ptr.add(4).copy_from_nonoverlapping(s.as_ptr(), len as usize);\n\
                                {P3}ptr.add((4 + len) as usize).write(0);"
                            )?;
                            format_args!("{P3}ptr = ptr.add(4 + roundup4(len + 1) as usize);")
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
            {P2}pub unsafe fn encode({arguments}{fmut}ptr: *mut u8) {{\n\
            {body}\
            {P2}}}\n\
            "
        );
    }

    fn generate_empty_encodable(&self, opcode: u16, cx: OpContext, o: &mut impl Write) {
        let OpContext { struct_name, .. } = cx;
        let fmtopcode = opcode_ne_bytes(opcode);

        self.generate_mod_header(opcode, &cx, o);
        writeln!(
            o,
            "\
            {P2}#[inline]\n\
            {P2}pub const fn encode(object_id: u32) -> [u8; 8] {{\n\
            {P2}    let mut buf = [0, 0, 0, 0, {fmtopcode}8, 0];\n\
            {P2}    *buf.first_chunk_mut().unwrap() = object_id.to_ne_bytes();\n\
            {P2}    buf\n\
            {P2}}}\n\n\
            {P2}#[doc(hidden)]\n\
            {P2}#[inline]\n\
            {P2}pub const fn new() -> {struct_name} {{\n\
            {P2}    {struct_name}\n\
            {P2}}}\n\n\
            {P2}pub struct {struct_name};\n\n\
            {P2}impl {struct_name} {{\n\
            {P2}    #[inline]\n\
            {P2}    pub const fn to_encoded(&self, object_id: u32) -> [u8; 8] {{\n\
            {P2}        encode(object_id)\n\
            {P2}    }}\n\
            {P2}}}"
        );
        self.generate_mod_trailer(o);
    }
}

impl Enum {
    pub fn generate(&self, o: &mut impl Write) {
        let name = StructName(self.name.as_str());
        let bitfield = self.bitfield;
        let entries = &self.entries;

        writeln!(o);
        if let Some(since) = self.since {
            writeln!(o, "{P1}/// since: {since}");
        }
        writeln!(o, "{P1}/// bitfield: {bitfield}");
        writeln!(o, "{P1}pub enum {name} {{");
        for entry in entries {
            let name = StructName(entry.name.as_str());
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
    fn data(&self) -> (u32, u32) {
        let mut fd_count = 0;
        let mut dynamic = 0;
        for arg in self.0 {
            fd_count += arg.is_fd() as u32;
            dynamic += (arg.is_dynamic_size() || arg.is_implicit_new_id()) as u32;
        }
        (fd_count, dynamic)
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

/// Some argument type require special handling.
enum ArgKind {
    Regular,
    /// implicit new_id.
    ImplNewId,
    /// dynamic length
    Dynamic,
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

    fn kind(&self) -> ArgKind {
        if self.is_implicit_new_id() {
            ArgKind::ImplNewId
        } else if self.is_dynamic_size() {
            ArgKind::Dynamic
        } else {
            ArgKind::Regular
        }
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

    /// Rust type with nullable variant.
    fn to_rust_type(&self) -> &'static str {
        match self.ty {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => if self.allow_null {
                "Option<&'a str>"
            } else {
                "&'a str"
            },
            Type::Array => "&'a [u8]",
            Type::Fd => "RawFd",
            Type::NewId => "u32",
            Type::Object => if self.allow_null {
                "u32"
            } else {
                "NonZeroU32"
            },
        }
    }

    fn to_rust_type_no_lifetime(&self) -> &'static str {
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

/// Format opcode to native endian bytes.
fn opcode_ne_bytes(opcode: u16) -> impl std::fmt::Display {
    std::fmt::from_fn(move |f| {
        for b in opcode.to_ne_bytes() {
            write!(f, "{b}, ")?;
        }
        Ok(())
    })
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
struct StructName<'a>(&'a str);

impl<'a> std::fmt::Display for StructName<'a> {
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

struct WlTypeDocs<'a>(&'a Arg);

impl<'a> std::fmt::Display for WlTypeDocs<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(enum_name) = &self.0.enum_name {
            write!(f, "{enum_name}")?;
            return Ok(());
        }
        let ty = match self.0.ty {
            Type::Int => "int",
            Type::Uint => "uint",
            Type::Fixed => "fixed",
            Type::String => "string",
            Type::Array => "array",
            Type::Fd => "fd",
            Type::NewId => "new_id",
            Type::Object => "object",
        };
        write!(f, "{ty}")?;
        if let Some(iface) = &self.0.interface {
            write!(f, "<{iface}>")?;
        }
        if self.0.allow_null {
            write!(f, " | null")?;
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
