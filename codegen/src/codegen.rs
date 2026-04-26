use crate::Write;
use crate::element::{Arg, Description, Entry, Enum, Interface, Op, Protocol};

const P1: &str = "    ";
const P2: &str = "        ";
const P3: &str = "            ";
const P4: &str = "                ";

// prelude
const TYPE_TRAIT: &str = "Type";
const ENCODE_TRAIT: &str = "Encode";

const PRELUDE: &str = "
#![warn(unused_imports)]
pub use super::{Array, Type, Encode};\n\
pub use std::os::fd::RawFd;\n\
pub use std::num::NonZeroU32;\n\
";

impl Protocol {
    pub fn generate_header(&self, o: &mut impl Write) {
        let Self {
            name,
            copyright,
            description,
        } = self;

        writeln!(o, "//! {name}");
        writeln!(o, "//!");

        if let Some(cp) = copyright {
            writeln!(o, "//! {cp}");
            writeln!(o, "//!");
        }
        if let Some(desc) = description {
            desc.generate_inner_doc("", o);
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
            description,
        } = self;

        if let Some(desc) = description {
            desc.generate("", o);
        }

        writeln!(
            o, "pub mod {name} {{\n\
            {P1}use super::*;\n\
            {P1}pub const VERSION: u32 = {version};\n\
            {P1}pub const FROZEN: bool = {frozen};\n\
        ");
    }
}

impl Op {
    pub fn generate(&self, opcode: u32, o: &mut impl Write) {
        let Op {
            name,
            description,
            kind,
            destructor,
            since,
            deprecated_since,
            args,
        } = self;

        let is_request = kind.is_request();
        let is_event = kind.is_event();

        writeln!(o);

        if let Some(desc) = description {
            desc.generate(P1, o);
        }
        if let Some(since) = since {
            writeln!(o, "{P1}///");
            writeln!(o, "{P1}/// {since}");
        }
        if let Some(dep_since) = deprecated_since {
            writeln!(o, "{P1}///");
            writeln!(o, "{P1}/// {dep_since}");
        }

        let name = name.to_camel_case();
        let lifetime = if Arg::need_lifetime(args) {
            format_args!("<'a>")
        } else {
            format_args!("")
        };

        writeln!(o, "{P1}pub struct {name}{lifetime} {{");
        for arg in args {
            let Arg {
                name,
                description,
                ty,
                interface,
                allow_null,
                enum_name,
                summary,
            } = arg;

            let has_header = description.is_some() || summary.is_some();
            let has_trailer = interface.is_some() || enum_name.is_some();

            // summary should not be used if a description is used.
            if let Some(desc) = description {
                desc.generate(P2, o);
            } else if let Some(sum) = summary {
                writeln!(o, "{P2}/// {sum}");
            }

            if has_header && has_trailer {
                writeln!(o, "{P2}///");
            }

            if let Some(iface) = interface {
                writeln!(o, "{P2}/// {iface}");
            }
            if let Some(enum_name) = enum_name {
                writeln!(o, "{P2}/// {enum_name}");
            }

            let ty_name = ty.to_rust_type(interface.as_deref());

            write!(o, "{P2}pub {name}: ");
            if *allow_null {
                writeln!(o, "Option<{ty_name}>,");
            } else {
                writeln!(o, "{ty_name},");
            }
        }
        writeln!(o, "{P1}}}");
        writeln!(o);

        // impl
        writeln!(o, "{P1}impl{lifetime} {name}{lifetime} {{");

        // ===== fn size() =====
        writeln!(o, "{P2}pub fn size(&self) -> usize {{");
        for (is_first, arg) in args.with_first() {
            if !is_first {
                write!(o, " + ");
            }
            write!(o, "{TYPE_TRAIT}::size(&self.{})", arg.name);
        }
        writeln!(o, "{P2}}}");
        writeln!(o);

        // ===== fn encode() =====
        writeln!(o, "{P2}pub fn encode(&self, buf: &mut [u8]) {{");
        match args.as_slice() {
            [] => writeln!(o, "{P3}let _ = buf;"),
            [arg] => writeln!(o, "{ENCODE_TRAIT}::encode(&self.{}, buf);", arg.name),
            _ => {
                writeln!(o, "{P3}if buf.len() != self.size() {{");
                writeln!(o, "{P3}    panic!(\"buffer should have the exact required length\");");
                writeln!(o, "{P3}}}");
                writeln!(o, "{P3}unsafe {{");
                for (is_last, arg) in args.with_last() {
                    let buf = if is_last {
                        "buf"
                    } else {
                        writeln!(o, "{P4}let (write, buf) = buf.split_at_mut_unchecked(Type::size(&self.{}));", arg.name);
                        "write"
                    };
                    writeln!(o, "{P4}{ENCODE_TRAIT}::encode_unchecked(&self.{}, {buf});", arg.name);
                }
                writeln!(o, "{P3}}}");
            },
        }
        writeln!(o, "{P2}}}");
        writeln!(o);

        // end impl
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

        writeln!(o);

        if let Some(desc) = description {
            desc.generate(P1, o);
            writeln!(o, "{P1}///");
        }
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
                summary,
                description,
                since,
                deprecated_since,
            } = entry;

            let has_header = description.is_some() || summary.is_some();
            let has_trailer = since.is_some() || deprecated_since.is_some();

            // summary should not be used if a description is used.
            if let Some(desc) = description {
                desc.generate(P2, o);
            } else if let Some(sum) = summary {
                writeln!(o, "{P2}/// {sum}");
            }

            if has_header && has_trailer {
                writeln!(o, "{P2}///");
            }

            if let Some(since) = since {
                writeln!(o, "{P2}/// since: {since}");
            }
            if let Some(dep_since) = deprecated_since {
                writeln!(o, "{P2}/// deprecated-since: {dep_since}");
            }

            let name = name.to_camel_case();

            writeln!(o, "{name} = {value},");
        }
        writeln!(o, "{P1}}}");
    }
}

impl Description {
    /// Write an inner doc comment.
    pub fn generate_inner_doc(&self, pad: &str, o: &mut impl Write) {
        let Self { summary, content } = self;
        if let Some(sum) = summary.as_deref() {
            writeln!(o, "{pad}//! {sum}");
            writeln!(o, "{pad}//!");
        }
        writeln!(o, "{pad}//! {content}");
    }

    /// Write an outer doc comment.
    pub fn generate(&self, pad: &str, o: &mut impl Write) {
        let Self { summary, content } = self;
        if let Some(sum) = summary.as_deref() {
            writeln!(o, "{pad}/// {sum}");
            writeln!(o, "{pad}///");
        }
        writeln!(o, "{pad}/// {content}");
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
