use std::env::args;
use std::fs::File;

use crate::parser::Parser;

mod parser;

// `/usr/share/wayland/wayland.dtd`
// <!ELEMENT protocol (copyright?, description?, interface+)>
//   <!ATTLIST protocol name CDATA #REQUIRED>
// <!ELEMENT copyright (#PCDATA)>
// <!ELEMENT interface (description?,(request|event|enum)+)>
//   <!ATTLIST interface name CDATA #REQUIRED>
//   <!ATTLIST interface version CDATA #REQUIRED>
// <!ELEMENT request (description?,arg*)>
//   <!ATTLIST request name CDATA #REQUIRED>
//   <!ATTLIST request type CDATA #IMPLIED>
//   <!ATTLIST request since CDATA #IMPLIED>
//   <!ATTLIST request deprecated-since CDATA #IMPLIED>
// <!ELEMENT event (description?,arg*)>
//   <!ATTLIST event name CDATA #REQUIRED>
//   <!ATTLIST event type CDATA #IMPLIED>
//   <!ATTLIST event since CDATA #IMPLIED>
//   <!ATTLIST event deprecated-since CDATA #IMPLIED>
// <!ELEMENT enum (description?,entry*)>
//   <!ATTLIST enum name CDATA #REQUIRED>
//   <!ATTLIST enum since CDATA #IMPLIED>
//   <!ATTLIST enum bitfield CDATA #IMPLIED>
// <!ELEMENT entry (description?)>
//   <!ATTLIST entry name CDATA #REQUIRED>
//   <!ATTLIST entry value CDATA #REQUIRED>
//   <!ATTLIST entry summary CDATA #IMPLIED>
//   <!ATTLIST entry since CDATA #IMPLIED>
//   <!ATTLIST entry deprecated-since CDATA #IMPLIED>
// <!ELEMENT arg (description?)>
//   <!ATTLIST arg name CDATA #REQUIRED>
//   <!ATTLIST arg type CDATA #REQUIRED>
//   <!ATTLIST arg summary CDATA #IMPLIED>
//   <!ATTLIST arg interface CDATA #IMPLIED>
//   <!ATTLIST arg allow-null CDATA #IMPLIED>
//   <!ATTLIST arg enum CDATA #IMPLIED>
// <!ELEMENT description (#PCDATA)>
//   <!ATTLIST description summary CDATA #REQUIRED>

// PCDATA is semantic data, it should be parsed
// CDATA is plain text, it should not be parsed

// !ELEMENT
// parenth contains childrens, in the same sequence, comma separated
// - `+`, at least one, or more
// - `*`, zero or more
// - `?`, zero or one
// - `|`, either/or
//

// <!ATTLIST element-name attribute-name attribute-type attribute-value>
// attribute are NOT in order, in `enum dnd_action`, `bitfield` appear before `since`
// #IMPLIED is optional

macro_rules! assert_atr {
    ($name:expr, $($tt:tt)*) => {
        assert!(matches!($name, $($tt)*), "unknown attribute: {}", f($name))
    };
}

fn main() {
    let Some(path) = args().nth(1) else {
        panic!("path arguments is required");
    };

    let mut parser = Parser::new(File::open(path).unwrap());
    let mut output = std::io::stdout().lock();

    // <!ELEMENT protocol (copyright?, description?, interface+)>
    let tag = parser.next_tag_assert("protocol");
    let name = tag.attrs().next_assert("name");
    writeln!(output, "//! {}", f(name));

    // copyright?
    if parser.next_tag_if("copyright").is_some() {
        // <!ELEMENT copyright (#PCDATA)>
        parser.next_closing_tag_assert("copyright");
    }

    // description?
    if let Some(tag) = parser.next_tag_if("description") {
        // <!ELEMENT description (#PCDATA)>
        //   <!ATTLIST description summary CDATA #REQUIRED>
        let mut attrs = tag.attrs();
        let summary = attrs.next_assert("summary");
        writeln!(output, "//!");
        writeln!(output, "//! {}", f(summary));
        let description = parser.next_plain();
        writeln!(output, "//!");
        writeln!(output, "//! {}", f(description));
        parser.next_closing_tag_assert("description");
    }

    loop {
        let tag = parser.peek_tag();
        let tag_name = tag.name();
        if tag_name == b"protocol" {
            break;
        }
        assert_eq!(tag_name, b"interface");
        interface(&mut parser, &mut output);
    }
}

fn interface<O: Write>(parser: &mut Parser, output: &mut O) {
    // <!ELEMENT interface (description?,(request|event|enum)+)>
    //   <!ATTLIST interface name CDATA #REQUIRED>
    //   <!ATTLIST interface version CDATA #REQUIRED>

    let tag = parser.next_tag_assert("interface");

    let mut attrs = tag.attrs();
    let name = attrs.next_assert("name").to_vec();
    let version = attrs.next_assert("version").to_vec();

    // ===== description? =====

    writeln!(output);
    process_description(parser, output, "");

    writeln!(output, "pub mod {} {{", f(&name));
    writeln!(output, "    /// {} version", f(&name));
    writeln!(output, "    pub const VERSION: u32 = {};", f(&version));

    // ===== (request|event|enum)+ =====

    let mut reqcode = 0;
    let mut evcode = 0;
    loop {
        let tag = parser.peek_tag();
        let name = tag.name();
        if name == b"interface" {
            break;
        }
        writeln!(output);
        match tag.name() {
            b"request" => {
                process_operation("request", reqcode, parser, output);
                reqcode += 1;
            }
            b"event" => {
                process_operation("event", evcode, parser, output);
                evcode += 1;
            }
            b"enum" => {
                process_enum(parser, output);
            }
            _ => unreachable!(),
        }
    }

    // close interface mod
    writeln!(output, "}}");

    parser.next_closing_tag_assert("interface");
}

fn process_description<O: Write>(parser: &mut Parser, output: &mut O, pad: &str) {
    // <!ELEMENT description (#PCDATA)>
    //   <!ATTLIST description summary CDATA #REQUIRED>

    let Some(tag) = parser.next_tag_if("description") else {
        return;
    };

    let is_self_close = tag.is_self_close();

    let mut attrs = tag.attrs();
    let summary = attrs.next_assert("summary").trim_ascii();

    writeln!(output, "{pad}/// {}", f(summary));
    writeln!(output, "{pad}///");

    let desc = parser.next_plain().trim_ascii();
    for line in std::io::BufRead::lines(desc).map(Result::unwrap) {
        let line = line.as_bytes().trim_ascii();
        let sp = if line.is_empty() { "" } else { " " };
        writeln!(output, "{pad}///{sp}{}", f(line));
    }

    // there is self closed description
    if is_self_close {
        return
    }

    parser.next_closing_tag_assert("description");
}

fn process_operation<O: Write>(op: &str, opcode: usize, parser: &mut Parser, output: &mut O) {
    // <!ELEMENT request (description?,arg*)>
    //   <!ATTLIST request name CDATA #REQUIRED>
    //   <!ATTLIST request type CDATA #IMPLIED>
    //   <!ATTLIST request since CDATA #IMPLIED>
    //   <!ATTLIST request deprecated-since CDATA #IMPLIED>
    // <!ELEMENT event (description?,arg*)>
    //   <!ATTLIST event name CDATA #REQUIRED>
    //   <!ATTLIST event type CDATA #IMPLIED>
    //   <!ATTLIST event since CDATA #IMPLIED>
    //   <!ATTLIST event deprecated-since CDATA #IMPLIED>

    let tag = parser.next_tag_assert(op);
    let mut attrs = tag.attrs();
    let name = attrs.next_assert("name");
    let name = if name == b"move" {
        // some request is a rust keyword
        b"r#move".to_vec()
    } else {
        name.to_vec()
    };
    let mut is_type_destructor = false;

    // need to be buffered because `description` needed before all the attributes, but appear after
    let buf_output = {
        let mut output = Vec::with_capacity(256);
        while let Some(attr) = attrs.try_next() {
            let atr_name = attr.name();
            match atr_name {
                b"type" => {
                    // `type` can only contains literal `destructor`
                    assert_eq!(attr.value(), b"destructor");
                    is_type_destructor = true;
                }
                b"since" | b"deprecated-since" => {
                    writeln!(output, "    ///");
                    writeln!(output, "    /// {}: {}", f(atr_name), f(attr.value()));
                }
                _ => unreachable!()
            }
        }
        output
    };

    process_description(parser, output, "    ");
    write!(output, "{}", f(&buf_output));

    let mut cop = op.as_bytes().to_vec();
    cop[0].make_ascii_uppercase();

    writeln!(output, "    pub mod {} {{", f(&name));
    writeln!(output, "        pub const KIND: Kind = Kind::{};", f(&cop));
    writeln!(output, "        pub const OPCODE: u32 = {opcode};");
    writeln!(output, "        pub const IS_TYPE_DESTRUCTOR: bool = {is_type_destructor};");

    write!(output, "        pub fn write(");
    if parser.peek_tag().name() == b"arg" {
        process_arg(parser, output);
    }
    while parser.peek_tag().name() == b"arg" {
        write!(output, ", ");
        process_arg(parser, output);
    }
    writeln!(output, ") {{");
    writeln!(output, "            todo!()");
    writeln!(output, "        }}");
    writeln!(output, "    }}");

    parser.next_closing_tag_assert(op);
}

/// TODO: argument `summary`, `allow-null`, and `enum` currently ignored
fn process_arg<O: Write>(parser: &mut Parser, output: &mut O) {
    // <!ELEMENT arg (description?)>
    //   <!ATTLIST arg name CDATA #REQUIRED>
    //   <!ATTLIST arg type CDATA #REQUIRED>
    //   <!ATTLIST arg summary CDATA #IMPLIED>
    //   <!ATTLIST arg interface CDATA #IMPLIED>
    //   <!ATTLIST arg allow-null CDATA #IMPLIED>
    //   <!ATTLIST arg enum CDATA #IMPLIED>

    let tag = parser.next_tag_assert("arg");
    assert!(
        tag.is_self_close(),
        "argument with description is not yet implemented: {}",
        f(tag.name())
    );

    let mut attrs = tag.attrs();
    let name = attrs.next_assert("name");
    let ty = attrs.next_assert("type");

    write!(output, "{}: {}", f(name), f(ty));

    while let Some(attr) = attrs.try_next() {
        match attr.name() {
            b"summary" => {}
            b"interface" => {}
            b"allow-null" => {}
            b"enum" => {}
            name => unknown_attribute(name),
        }
    }
}

fn process_enum<O: Write>(parser: &mut Parser, output: &mut O) {
    // <!ELEMENT enum (description?,entry*)>
    //   <!ATTLIST enum name CDATA #REQUIRED>
    //   <!ATTLIST enum since CDATA #IMPLIED>
    //   <!ATTLIST enum bitfield CDATA #IMPLIED>

    let tag = parser.next_tag_assert("enum");
    let mut attrs = tag.attrs();
    let name = attrs.next_assert("name");

    // need to be buffered because `description` needed before all the attributes, but appear after
    let buf_output = {
        let mut output = Vec::with_capacity(256);
        while let Some(attr) = attrs.try_next() {
            let atr_name = attr.name();
            assert_atr!(atr_name, b"since" | b"bitfield");
            writeln!(output, "    ///");
            writeln!(output, "    /// {}: {}", f(atr_name), f(attr.value()));
        }
        writeln!(output, "    #[derive(Debug)]");
        writeln!(output, "    pub enum {} {{", f(name));
        output
    };

    // some enum does not have description
    process_description(parser, output, "    ");
    write!(output, "{}", f(&buf_output));

    while parser.peek_tag().name() == b"entry" {
        process_entry(parser, output);
    }

    // close enum
    writeln!(output, "    }}");

    parser.next_closing_tag_assert("enum");
}

fn process_entry<O: Write>(parser: &mut Parser, output: &mut O) {
    // <!ELEMENT entry (description?)>
    //   <!ATTLIST entry name CDATA #REQUIRED>
    //   <!ATTLIST entry value CDATA #REQUIRED>
    //   <!ATTLIST entry summary CDATA #IMPLIED>
    //   <!ATTLIST entry since CDATA #IMPLIED>
    //   <!ATTLIST entry deprecated-since CDATA #IMPLIED>
    const PAD: &str = "        ";

    let tag = parser.next_tag_assert("entry");
    assert!(
        tag.is_self_close(),
        "enum entry with description is not yet implemented: {}",
        f(tag.name())
    );

    let mut attrs = tag.attrs();
    let name = {
        let mut name = attrs.next_assert("name").to_vec();
        // some variant is a rust keyword
        name[0].make_ascii_uppercase();
        // some variant only contains digit
        if name.iter().all(|e| e.is_ascii_digit()) {
            name.insert(0, b'D');
        }
        name
    };
    let value = attrs.next_assert("value");

    while let Some(attr) = attrs.try_next() {
        let atr_name = attr.name();
        assert_atr!(atr_name, b"summary" | b"since" | b"deprecated-since");

        write!(output, "{PAD}///");
        if atr_name != b"summary" {
            write!(output, "\n{PAD}/// {}:", f(atr_name));
        }
        writeln!(output, " {}", f(attr.value()));
    }
    writeln!(output, "{PAD}{} = {},", f(&name), f(value));
}

// ===== Util =====

fn f(bytes: &[u8]) -> &str {
    // SAFETY: the parser guarantee that prolog is `encoding="UTF-8"`
    unsafe { str::from_utf8_unchecked(bytes) }
}

fn unknown_attribute(name: &[u8]) -> ! {
    panic!("unknown attribute: {:?}", str::from_utf8(name))
}

trait Write {
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>);
}

impl<W: std::io::Write> Write for W {
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        std::io::Write::write_fmt(self, args).unwrap();
    }
}

trait ParserAssert {
    fn next_tag_assert(&mut self, name: &str) -> parser::Tag<'_>;

    fn next_closing_tag_assert(&mut self, name: &str) -> parser::Tag<'_>;

    fn next_tag_if(&mut self, name: &str) -> Option<parser::Tag<'_>>;
}

impl ParserAssert for Parser {
    fn next_tag_assert(&mut self, name: &str) -> parser::Tag<'_> {
        let tag = self.next_tag();
        assert_eq!(tag.name(), name.as_bytes());
        tag
    }

    fn next_closing_tag_assert(&mut self, name: &str) -> parser::Tag<'_> {
        let tag = self.next_tag();
        assert_eq!(tag.name(), name.as_bytes());
        assert!(tag.is_closing());
        tag
    }

    fn next_tag_if(&mut self, name: &str) -> Option<parser::Tag<'_>> {
        if self.peek_tag().name() == name.as_bytes() {
            Some(self.next_tag())
        } else {
            None
        }
    }
}

trait AttrAssert<'a> {
    fn next_assert(&mut self, name: &str) -> &'a [u8];
}

impl<'a> AttrAssert<'a> for parser::Attrs<'a> {
    fn next_assert(&mut self, name: &str) -> &'a [u8] {
        let attr = self.next();
        assert_eq!(attr.name(), name.as_bytes());
        attr.value()
    }
}

