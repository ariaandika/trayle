use std::env::args;
use std::fs::File;

use crate::buffer::FileBuffer;
use crate::parser::Parser;

mod buffer;
mod parser;
mod element;
mod codegen;

fn main() {
    let Some(path) = args().nth(1) else {
        eprintln!("Error: file path argument is required");
        std::process::exit(1);
    };

    let mut file_buffer = FileBuffer::new(File::open(path).unwrap());
    file_buffer.read();

    let mut parser = Parser::new(file_buffer);
    let mut output = std::io::stdout().lock();

    parse_protocol(&mut parser, &mut output);

    while parse_interface(&mut parser, &mut output) { }
}

fn parse_protocol(parser: &mut Parser, output: &mut impl Write) {
    parser.assert_prolog("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");

    // <!ELEMENT protocol (copyright?, description?, interface+)>
    let (tag, _) = parser.next_tag("protocol");
    let name = tag.attrs().next("name").value();

    // copyright?
    let copyright = if let Some((_, mut content)) = parser.next_tag_if("copyright") {
        let (tag, _) = parser.next_tag("copyright");
        assert!(tag.is_closing());
        content.trim_ascii();
        Some(content)
    } else {
        None
    };

    // description?
    let description = parse_description(parser);

    element::Protocol {
        name,
        copyright,
        description,
    }.generate_header(output);
}

fn parse_description(parser: &mut Parser) -> Option<element::Description> {
    // <!ELEMENT description (#PCDATA)>
    //   <!ATTLIST description summary CDATA #REQUIRED>
    if let Some((tag, content)) = parser.next_tag_if("description") {
        let is_self_close = tag.is_self_close();
        let summary = tag.attrs().next("summary").value();
        // some description is self closing tag
        if !is_self_close {
            let (tag, _) = parser.next_tag("description");
            assert!(tag.is_closing());
        }
        Some(element::Description { summary, content })
    } else {
        None
    }
}

fn parse_interface(parser: &mut Parser, output: &mut impl Write) -> bool {
    // <!ELEMENT interface (description?,(request|event|enum)+)>
    //   <!ATTLIST interface name CDATA #REQUIRED>
    //   <!ATTLIST interface version CDATA #REQUIRED>
    //   <!ATTLIST interface frozen CDATA #IMPLIED>
    let Some((tag, _)) = parser.next_tag_if("interface") else {
        return false;
    };
    let mut attrs = tag.attrs();
    let name = attrs.next("name").value();
    let version = atou(attrs.next("version").value_str().as_bytes());
    let frozen = match attrs.next_if("frozen") {
        Some(attr) => {
            assert_eq!(attr.value_str(), "true");
            true
        }
        None => false,
    };

    let description = if let Some((tag, content)) = parser.next_tag_if("description") {
        let summary = tag.attrs().next("summary").value();
        let (tag, _) = parser.next_tag("description");
        assert!(tag.is_closing());
        Some(element::Description { summary, content })
    } else {
        None
    };

    element::Interface {
        name,
        description,
        version,
        frozen,
    }
    .generate_header(output);

    // ===== request/event =====

    let mut state = InterfaceOpCode::new();
    loop {
        match parser.peek() {
            b"request" => parse_operation(
                "request",
                element::OpKind::Request,
                state.request(),
                parser,
                output,
            ),
            b"event" => parse_operation(
                "event",
                element::OpKind::Event,
                state.event(),
                parser,
                output,
            ),
            b"enum" => parse_enum(parser, output),
            _ => break,
        }
    }

    // ===== end request/event =====

    let (tag, _) = parser.next_tag("interface");
    assert!(tag.is_closing());
    element::Interface::generate_trailer(output);

    true
}

struct InterfaceOpCode {
    request: u16,
    event: u16,
}

impl InterfaceOpCode {
    fn new() -> Self {
        Self { request: 0, event: 0 }
    }

    fn request(&mut self) -> u16 {
        let r = self.request;
        self.request += 1;
        r
    }

    fn event(&mut self) -> u16 {
        let e = self.event;
        self.event += 1;
        e
    }
}

fn parse_operation(
    tag_name: &str,
    kind: element::OpKind,
    opcode: u16,
    parser: &mut Parser,
    output: &mut impl Write,
) {
    // <!ELEMENT request (description?,arg*)>
    // <!ELEMENT event (description?,arg*)>
    //   <!ATTLIST _ name CDATA #REQUIRED>
    //   <!ATTLIST _ type CDATA #IMPLIED>
    //   <!ATTLIST _ since CDATA #IMPLIED>
    //   <!ATTLIST _ deprecated-since CDATA #IMPLIED>
    let (tag, _) = parser.next_tag(tag_name);
    let mut attrs = tag.attrs();
    let name = attrs.next("name").value();
    let mut destructor = false;
    let mut since = None;
    let mut deprecated_since = None;

    while let Some(attr) = attrs.peek() {
        match attr {
            "type" => {
                let attr = attrs.next("type");
                assert_eq!(attr.value_str(), "destructor");
                destructor = true;
            }
            "since" => {
                let attr = attrs.next("since");
                since = Some(atou(attr.value_str().as_bytes()));
            }
            "deprecated-since" => {
                let attr = attrs.next("deprecated-since");
                deprecated_since = Some(atou(attr.value_str().as_bytes()));
            }
            name => unreachable!("unknown attribute: `{name}`"),
        }
    }

    let description = parse_description(parser);
    let mut args = vec![];

    while let Some((tag, _)) = parser.next_tag_if("arg") {
        // <!ELEMENT arg (description?)>
        //   <!ATTLIST arg name CDATA #REQUIRED>
        //   <!ATTLIST arg type CDATA #REQUIRED>
        //   <!ATTLIST arg summary CDATA #IMPLIED>
        //   <!ATTLIST arg interface CDATA #IMPLIED>
        //   <!ATTLIST arg allow-null CDATA #IMPLIED>
        //   <!ATTLIST arg enum CDATA #IMPLIED>
        let mut attrs = tag.attrs();
        let name = attrs.next("name").value();
        let ty = element::Type::from_wl_type(attrs.next("type").value_str());
        let mut summary = None;
        let mut interface = None;
        let mut allow_null = false;
        let mut enum_name = None;

        while let Some(attr) = attrs.peek() {
            match attr {
                "summary" => {
                    let attr = attrs.next("summary");
                    summary = Some(attr.value());
                }
                "interface" => {
                    let attr = attrs.next("interface");
                    interface = Some(attr.value());
                }
                "enum" => {
                    let attr = attrs.next("enum");
                    enum_name = Some(attr.value());
                }
                "allow-null" => {
                    let attr = attrs.next("allow-null");
                    allow_null = match attr.value_str() {
                        "true" => true,
                        "false" => false,
                        _ => unreachable!(),
                    };
                }
                name => unreachable!("unknown attribute: `{name}`"),
            }
        }
        let description = parse_description(parser);
        args.push(element::Arg {
            name,
            ty,
            interface,
            allow_null,
            enum_name,
            summary,
            description,
        });
    }

    let (tag, _) = parser.next_tag(kind.as_str());
    assert!(tag.is_closing());

    element::Op {
        name,
        kind,
        destructor,
        since,
        deprecated_since,
        description,
        args,
    }.generate(opcode, output);
}

fn parse_enum(parser: &mut Parser, output: &mut impl Write) {
    // <!ELEMENT enum (description?,entry*)>
    //   <!ATTLIST enum name CDATA #REQUIRED>
    //   <!ATTLIST enum since CDATA #IMPLIED>
    //   <!ATTLIST enum bitfield CDATA #IMPLIED>
    let (tag, _) = parser.next_tag("enum");

    let mut attrs = tag.attrs();
    let name = attrs.next("name").value();
    let mut since = None;
    let mut bitfield = false;

    while let Some(attr) = attrs.peek() {
        match attr {
            "since" => {
                let attr = attrs.next("since");
                since = Some(atou(attr.value_str().as_bytes()));
            }
            "bitfield" => {
                let attr = attrs.next("bitfield");
                bitfield = match attr.value_str() {
                    "true" => true,
                    "false" => false,
                    _ => unreachable!()
                };
            }
            name => unreachable!("unknown attribute: `{name}`"),
        }
    }

    let description = parse_description(parser);
    let mut entries = vec![];

    while let Some((tag, _)) = parser.next_tag_if("entry") {
        // <!ELEMENT entry (description?)>
        //   <!ATTLIST entry name CDATA #REQUIRED>
        //   <!ATTLIST entry value CDATA #REQUIRED>
        //   <!ATTLIST entry summary CDATA #IMPLIED>
        //   <!ATTLIST entry since CDATA #IMPLIED>
        //   <!ATTLIST entry deprecated-since CDATA #IMPLIED>
        let mut attrs = tag.attrs();
        let name = attrs.next("name").value();
        let value = attrs.next("value").value();
        let mut summary = None;
        let mut since = None;
        let mut deprecated_since = None;

        while let Some(attr) = attrs.peek() {
            match attr {
                "summary" => {
                    let attr = attrs.next("summary");
                    summary = Some(attr.value());
                }
                "since" => {
                    let attr = attrs.next("since");
                    since = Some(atou(attr.value_str().as_bytes()));
                }
                "deprecated-since" => {
                    let attr = attrs.next("deprecated-since");
                    deprecated_since = Some(atou(attr.value_str().as_bytes()));
                }
                name => unreachable!("unknown attribute: `{name}`"),
            }
        }
        let description = parse_description(parser);
        entries.push(element::Entry {
            name,
            value,
            summary,
            description,
            since,
            deprecated_since,
        });
    }

    let (tag, _) = parser.next_tag("enum");
    assert!(tag.is_closing());

    let enum_ = element::Enum {
        name,
        description,
        since,
        bitfield,
        entries,
    };
    write!(output, "{enum_}");
}

// ===== Util =====

fn atou(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0u32, |acc, next| match next.wrapping_sub(b'0') {
            b @ 0..=9 => acc.wrapping_mul(10).wrapping_add(b as _),
            _ => panic!("non integer"),
        })
}

trait Write {
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>);
}

impl<W: std::io::Write> Write for W {
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) {
        std::io::Write::write_fmt(self, args).unwrap();
    }
}
