use std::env::args;
use std::fs::File;

use buffer::FileBuffer;
use element::*;
use parser::Parser;

mod buffer;
mod parser;
mod element;
mod codegen;

macro_rules! parse_attr {
    (
        $parser:ident, $tag:ident,
        $($attrid:ident $attr:literal),*
        $([
            $($oaid:ident $oas:literal),*
        ])?
    ) => {
        let mut attrs = $tag.attrs();
        $(
            let $attrid = attrs.next($attr).value();
        )*
        $(
            $(
                let mut $oaid = None;
            )*
            while let Some(attr) = attrs.peek() {
                match attr {
                    $(
                        $oas => {
                            $oaid = Some(attrs.next($oas).value());
                        }
                    )*
                    name => panic!("unknown attribute: `{name}`"),
                }
            }
        )?
    };
    (@2 $oattr:ident $osattr:literal) => {
        let mut $oattr = None;
    }
}

fn main() {
    let Some(path) = args().nth(1) else {
        eprintln!("Error: file path argument is required");
        std::process::exit(1);
    };

    let mut file_buffer = FileBuffer::new(File::open(path).unwrap());
    file_buffer.read();

    let mut parser = Parser::new(file_buffer);
    let mut output = std::io::stdout().lock();

    parser.assert_prolog("<?xml version=\"1.0\" encoding=\"UTF-8\"?>");

    parse_protocol(&mut parser, &mut output);

    while parse_interface(&mut parser, &mut output) {
        let mut request_opcode = 0;
        let mut event_opcode = 0;
        loop {
            let opcode;
            let (name, kind) = match parser.peek() {
                b"request" => {
                    opcode = request_opcode;
                    request_opcode += 1;
                    ("request", OpKind::Request)
                },
                b"event" => {
                    opcode = event_opcode;
                    event_opcode += 1;
                    ("event", OpKind::Event)
                },
                b"enum" => {
                    parse_enum(&mut parser, &mut output);
                    continue;
                },
                _ => break,
            };
            parse_operation(name, kind, opcode, &mut parser, &mut output);
        }

        parser.next_closing_tag("interface");
        Interface::generate_trailer(&mut output);
    }
}

fn parse_protocol(parser: &mut Parser, output: &mut impl Write) {
    // <!ELEMENT protocol (copyright?, description?, interface+)>
    //   <!ATTLIST protocol name CDATA #REQUIRED>
    let (tag, _) = parser.next_tag("protocol");
    let name = tag.attrs().next("name").value();

    // copyright?
    let copyright = parser.next_tag_if("copyright").map(|(_, mut content)| {
        parser.next_closing_tag("copyright");
        content.trim_ascii_mut();
        content
    });

    // description?
    let description = parse_description(parser);

    Protocol {
        name,
        copyright,
        description,
    }
    .generate_header(output);
}

fn parse_description(parser: &mut Parser) -> Option<Description> {
    // <!ELEMENT description (#PCDATA)>
    //   <!ATTLIST description summary CDATA #REQUIRED>
    let (tag, content) = parser.next_tag_if("description")?;
    let is_self_close = tag.is_self_close();
    let summary = tag.attrs().next("summary").value();
    // some description is self closing tag
    if !is_self_close {
        parser.next_closing_tag("description");
    }
    Some(Description { summary, content })
}

fn parse_interface(parser: &mut Parser, output: &mut impl Write) -> bool {
    // <!ELEMENT interface (description?,(request|event|enum)+)>
    //   <!ATTLIST interface name CDATA #REQUIRED>
    //   <!ATTLIST interface version CDATA #REQUIRED>
    //   <!ATTLIST interface frozen CDATA #IMPLIED>
    let Some((tag, _)) = parser.next_tag_if("interface") else {
        return false;
    };
    parse_attr!(parser, tag,
        name "name",
        version "version"
        [frozen "frozen"]
    );
    let version = atou(version.as_bytes());
    let frozen = frozen.inspect(|e|assert_eq!(e.as_str(), "true")).is_some();

    let description = parse_description(parser);

    Interface {
        name,
        description,
        version,
        frozen,
    }
    .generate_header(output);

    true
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
    parse_attr!(parser, tag,
        name "name"
        [
            ty "type",
            since "since",
            dep_since "deprecated-since"
        ]
    );

    ty.as_ref().inspect(|e|assert_eq!(e.as_str(), "destructor"));

    let destructor = ty.is_some();
    let since = since.map(|e|atou(e.as_bytes()));
    let deprecated_since = dep_since.map(|e|atou(e.as_bytes()));

    let description = parse_description(parser);

    // ===== Arguments =====
    let mut args = Vec::with_capacity(8);

    while let Some((tag, _)) = parser.next_tag_if("arg") {
        // <!ELEMENT arg (description?)>
        //   <!ATTLIST arg name CDATA #REQUIRED>
        //   <!ATTLIST arg type CDATA #REQUIRED>
        //   <!ATTLIST arg summary CDATA #IMPLIED>
        //   <!ATTLIST arg interface CDATA #IMPLIED>
        //   <!ATTLIST arg allow-null CDATA #IMPLIED>
        //   <!ATTLIST arg enum CDATA #IMPLIED>
        parse_attr!(parser, tag,
            name "name",
            ty "type"
            [
                summary "summary",
                interface "interface",
                enum_name "enum",
                allow_null "allow-null"
            ]
        );

        let ty = Type::from_wl_type(ty.as_str());
        // it said interface must be specified when `type=object`, but the first event in the core
        // protocol have interface-less object, breh
        //
        // if matches!(ty, Type::Object) {
        //     assert!(interface.is_some(), "type `object` should have `interface` ({tag_name}.{name})");
        // }
        if interface.is_some() {
            assert!(matches!(ty, Type::Object | Type::NewId));
        }
        let allow_null = allow_null.map(|e|e.as_str().parse().expect("invalid `allow-null`")).unwrap_or(false);
        if enum_name.is_some() {
            assert!(matches!(ty, Type::Uint | Type::Int));
        }

        let description = parse_description(parser);

        args.push(Arg {
            name,
            ty,
            interface,
            allow_null,
            enum_name,
            summary,
            description,
        });
    }

    parser.next_closing_tag(kind.as_str());

    Op {
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

    parse_attr!(parser, tag,
        name "name"
        [
            since "since",
            bitfield "bitfield"
        ]
    );
    let since = since.map(|e|atou(e.as_bytes()));
    let bitfield = bitfield.map(|e|e.parse().expect("invalid `bitfield`")).unwrap_or(false);

    let description = parse_description(parser);
    let mut entries = vec![];
    while let Some((tag, _)) = parser.next_tag_if("entry") {
        // <!ELEMENT entry (description?)>
        //   <!ATTLIST entry name CDATA #REQUIRED>
        //   <!ATTLIST entry value CDATA #REQUIRED>
        //   <!ATTLIST entry summary CDATA #IMPLIED>
        //   <!ATTLIST entry since CDATA #IMPLIED>
        //   <!ATTLIST entry deprecated-since CDATA #IMPLIED>
        parse_attr!(parser, tag,
            name "name",
            value "value"
            [
                summary "summary",
                since "since",
                dep_since "deprecated-since"
            ]
        );
        let since = since.map(|e|atou(e.as_bytes()));
        let deprecated_since = dep_since.map(|e|atou(e.as_bytes()));

        let description = parse_description(parser);
        entries.push(Entry {
            name,
            value,
            summary,
            description,
            since,
            deprecated_since,
        });
    }

    parser.next_closing_tag("enum");

    let enum_ = Enum {
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
