use std::env::args;
use std::fs::File;
use std::io::Write;

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

fn main() {
    let Some(path) = args().nth(1) else {
        panic!("path arguments is required");
    };

    let mut parser = Parser::new(File::open(path).unwrap());

    // <!ELEMENT protocol (copyright?, description?, interface+)>
    parser.next_tag_assert("protocol");

    // copyright?
    if parser.next_tag_if("copyright").is_some() {
        // <!ELEMENT copyright (#PCDATA)>
        parser.next_closing_tag_assert("copyright");
    }

    // description?
    if parser.next_tag_if("description").is_some() {
        // <!ELEMENT description (#PCDATA)>
        //   <!ATTLIST description summary CDATA #REQUIRED>
        parser.next_closing_tag_assert("description");
    }

    let mut stdout = std::io::stdout().lock();
    loop {
        let tag = parser.peek_tag();
        let tag_name = tag.name();
        if tag_name == b"protocol" {
            break;
        }
        assert_eq!(tag_name, b"interface");
        interface(&mut parser, &mut stdout);
    }
}

fn interface<O: Write>(parser: &mut Parser, mut output: O) {
    // <!ELEMENT interface (description?,(request|event|enum)+)>
    //   <!ATTLIST interface name CDATA #REQUIRED>
    //   <!ATTLIST interface version CDATA #REQUIRED>

    let tag = parser.next_tag_assert("interface");

    let mut attrs = tag.attrs();
    let name = attrs.next_assert("name").to_vec();
    let version = attrs.next_assert("version").to_vec();

    // ===== description? =====

    process_description(parser, &mut output, "");

    output.write_all(b"pub mod ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" {\n").unwrap();
    output.write_all(b"    /// ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" version\n    pub const VERSION: u32 = ").unwrap();
    output.write_all(&version).unwrap();
    output.write_all(b";\n\n").unwrap();

    // ===== (request|event|enum)+ =====

    let mut reqcode = 0;
    let mut evcode = 0;

    loop {
        let tag = parser.peek_tag();
        match tag.name() {
            b"request" => {
                process_operation("request", reqcode, parser, &mut output);
                reqcode += 1;
            }
            b"event" => {
                process_operation("event", evcode, parser, &mut output);
                evcode += 1;
            }
            b"enum" => {
                process_enum(parser, &mut output);
            }
            b"interface" => break,
            name => panic!("unknown interface property: {}", str::from_utf8(name).unwrap()),
        }
        output.write_all(b"\n").unwrap();
    }

    parser.next_closing_tag_assert("interface");

    // close interface mod
    output.write_all(b"}\n\n").unwrap();
}

fn process_description<O: Write>(parser: &mut Parser, mut output: O, pad: &str) {
    // <!ELEMENT description (#PCDATA)>
    //   <!ATTLIST description summary CDATA #REQUIRED>

    let Some(tag) = parser.next_tag_if("description") else {
        return;
    };

    let is_self_close = tag.is_self_close();

    let mut attrs = tag.attrs();
    let summary = attrs.next_assert("summary").trim_ascii();

    output.write_all(pad.as_bytes()).unwrap();
    output.write_all(b"/// ").unwrap();
    output.write_all(summary).unwrap();
    output.write_all(b"\n").unwrap();
    output.write_all(pad.as_bytes()).unwrap();
    output.write_all(b"///\n").unwrap();

    let desc = parser.next_plain().trim_ascii();

    for line in std::io::BufRead::lines(desc) {
        let line = line.unwrap();
        let line = line.as_bytes().trim_ascii();
        output.write_all(pad.as_bytes()).unwrap();
        output.write_all(b"/// ").unwrap();
        output.write_all(line).unwrap();
        output.write_all(b"\n").unwrap();
    }

    // there is self closed description
    if is_self_close {
        return
    }

    parser.next_closing_tag_assert("description");
}

fn process_operation<O: Write>(op: &str, opcode: usize, parser: &mut Parser, mut output: O) {
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

    // ===== description? =====

    process_description(parser, &mut output, "    ");

    output.write_all(b"    pub mod ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" {\n").unwrap();

    // ===== kind =====

    let mut cop = op.as_bytes().to_vec();
    cop[0].make_ascii_uppercase();
    output.write_all(b"        pub const KIND: Kind = Kind::").unwrap();
    output.write_all(&cop).unwrap();
    output.write_all(b";\n").unwrap();

    // ===== opcode =====

    output.write_all(b"        pub const OPCODE: u32 = ").unwrap();
    output.write_fmt(format_args!("{opcode}")).unwrap();
    output.write_all(b";\n").unwrap();

    // ===== arguments =====

    output.write_all(b"        pub fn write(").unwrap();

    loop {
        if parser.peek_tag().name() != b"arg" {
            break;
        }
        process_arg(parser, &mut output);
    }

    let tag = parser.next_tag();
    assert_eq!(tag.name(), op.as_bytes());
    assert!(tag.is_closing());

    output.write_all(b") { todo!() }\n").unwrap();

    // ===== close mod =====
    output.write_all(b"    }\n").unwrap();
}

fn process_arg<O: Write>(parser: &mut Parser, mut output: O) {
    // <!ELEMENT arg (description?)>
    //   <!ATTLIST arg name CDATA #REQUIRED>
    //   <!ATTLIST arg type CDATA #REQUIRED>
    //   <!ATTLIST arg summary CDATA #IMPLIED>
    //   <!ATTLIST arg interface CDATA #IMPLIED>
    //   <!ATTLIST arg allow-null CDATA #IMPLIED>
    //   <!ATTLIST arg enum CDATA #IMPLIED>

    let tag = parser.next_tag();
    assert_eq!(tag.name(), b"arg");

    let mut attrs = tag.attrs();
    let attr = attrs.next();
    assert_eq!(attr.name(), b"name");

    output.write_all(attr.value()).unwrap();
    output.write_all(b": ").unwrap();

    let attr = attrs.next();
    assert_eq!(attr.name(), b"type");

    output.write_all(attr.value()).unwrap();

    let attr = attrs.peek();
    if attr.filter(|e|e.name() == b"summary").is_some() {
        // TODO:
        attrs.next();
    }

    let attr = attrs.peek();
    if attr.filter(|e|e.name() == b"interface").is_some() {
        let attr = attrs.next();
        output.write_all(b"<").unwrap();
        output.write_all(attr.value()).unwrap();
        output.write_all(b">").unwrap();
    }

    output.write_all(b",").unwrap();

    let attr = attrs.peek();
    if attr.filter(|e|e.name() == b"allow-null").is_some() {
        // TODO:
        attrs.next();
    }
}

fn process_enum<O: Write>(parser: &mut Parser, mut output: O) {
    // <!ELEMENT enum (description?,entry*)>
    //   <!ATTLIST enum name CDATA #REQUIRED>
    //   <!ATTLIST enum since CDATA #IMPLIED>
    //   <!ATTLIST enum bitfield CDATA #IMPLIED>

    let tag = parser.next_tag_assert("enum");

    let mut attrs = tag.attrs();

    let name = attrs.next_assert("name").to_vec();
    let since = attrs.next_if("since").map(<[u8]>::to_vec);
    let bitfield = attrs.next_if("bitfield").map(<[u8]>::to_vec);

    // some enum does not have description
    process_description(parser, &mut output, "    ");

    if let Some(since) = since {
        output.write_all(b"    ///\n    /// since: ").unwrap();
        output.write_all(&since).unwrap();
        output.write_all(b"\n").unwrap();
    }

    if let Some(bitfield) = bitfield {
        output.write_all(b"    ///\n    /// bitfield: ").unwrap();
        output.write_all(&bitfield).unwrap();
        output.write_all(b"\n").unwrap();
    }

    output.write_all(b"    #[derive(Debug)]\n    pub enum ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" {\n").unwrap();

    loop {
        if parser.peek_tag().name() != b"entry" {
            break;
        }
        process_entry(parser, &mut output);
    }

    let tag = parser.next_tag();
    assert_eq!(tag.name(), b"enum");
    assert!(tag.is_closing());

    output.write_all(b"    }\n").unwrap();
}

fn process_entry<O: Write>(parser: &mut Parser, mut output: O) {
    // <!ELEMENT entry (description?)>
    //   <!ATTLIST entry name CDATA #REQUIRED>
    //   <!ATTLIST entry value CDATA #REQUIRED>
    //   <!ATTLIST entry summary CDATA #IMPLIED>
    //   <!ATTLIST entry since CDATA #IMPLIED>
    //   <!ATTLIST entry deprecated-since CDATA #IMPLIED>

    let tag = parser.next_tag_assert("entry");

    let mut attrs = tag.attrs();

    let name = {
        let mut name = attrs.next_assert("name").to_vec();
        // some variant is a rust keyword
        name[0].make_ascii_uppercase();
        // some variant only contains digit
        if name.iter().all(|e|e.is_ascii_digit()) {
            name.insert(0, b'd');
        }
        name
    };
    let value = attrs.next_assert("value");

    if let Some(summary) = attrs.next_if("summary") {
        output.write_all(b"        /// ").unwrap();
        output.write_all(summary).unwrap();
        output.write_all(b"\n").unwrap();
    }

    if let Some(since) = attrs.next_if("since") {
        output.write_all(b"        ///\n        /// since: ").unwrap();
        output.write_all(since).unwrap();
        output.write_all(b"\n").unwrap();
    }

    if let Some(dep_since) = attrs.next_if("deprecated-since") {
        output.write_all(b"\n        /// deprecated-since: ").unwrap();
        output.write_all(dep_since).unwrap();
        output.write_all(b"\n").unwrap();
    }

    output.write_all(b"        ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" = ").unwrap();
    output.write_all(value).unwrap();
    output.write_all(b",\n").unwrap();
}

// ===== Util =====

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

    fn next_if(&mut self, name: &str) -> Option<&'a [u8]>;
}

impl<'a> AttrAssert<'a> for parser::Attrs<'a> {
    fn next_assert(&mut self, name: &str) -> &'a [u8] {
        let attr = self.next();
        assert_eq!(attr.name(), name.as_bytes());
        attr.value()
    }

    fn next_if(&mut self, name: &str) -> Option<&'a [u8]> {
        self.peek()
            .filter(|e| e.name() == name.as_bytes())
            .map(|_| self.next().value())
    }
}

