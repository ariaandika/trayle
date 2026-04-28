use std::env::args;
use std::fs::File;
use std::task::Poll::{self, *};
use std::task::ready;

use crate::buffer::FileBuffer;
use crate::parser::Parser;

mod parser;
mod buffer;
mod element;
mod codegen;

// NOTE:
//
// some of rule in dtd file and the book is different, this codegen follows the dtd file
//
// parenth contains childrens, in the same sequence, comma separated
// - `+`, at least one, or more
// - `*`, zero or more
// - `?`, zero or one
// - `|`, either/or
//
// <!ATTLIST element-name attribute-name attribute-type attribute-value>
// attribute are NOT in order, in `enum dnd_action`, `bitfield` appear before `since`
// #IMPLIED is optional

macro_rules! advance_buffer {
    ($parser:ident, $bytes:ident, $buffer:ident) => {
        let last = $parser.as_bytes().first().expect("what are the odds");
        let len = $bytes.element_offset(last).unwrap();
        $buffer.advance(len);
    };
}

fn main() {
    let Some(path) = args().nth(1) else {
        eprintln!("error: protocol file path is required");
        eprintln!();
        eprintln!("available files:");

        const WAYLAND: &str = "/usr/share/wayland/wayland.xml";
        if std::path::Path::new(WAYLAND).exists() {
            eprintln!("{WAYLAND}");
        }

        let _ = std::process::Command::new("find")
            .args(["/usr/share/wayland-protocols/stable", "-type", "f"])
            .status();
        std::process::exit(1);
    };

    let mut buffer = FileBuffer::new(File::open(path).unwrap());
    let mut output = std::io::stdout().lock();

    loop {
        buffer.read();
        if parse_protocol(&mut buffer, &mut output).is_ready() {
            break
        }
    }

    loop {
        let Ready(ok) = parse_interface(&mut buffer, &mut output) else {
            buffer.read();
            continue;
        };
        if !ok {
            break;
        }
    }
}

fn parse_protocol(buffer: &mut FileBuffer, output: &mut impl Write) -> Poll<()> {
    let bytes = buffer.as_bytes();
    let mut parser = Parser::new(bytes);

    ready!(parser.assert_prolog("<?xml version=\"1.0\" encoding=\"UTF-8\"?>"));

    // <!ELEMENT protocol (copyright?, description?, interface+)>
    let (tag, _) = ready!(parser.next_tag("protocol"));
    let name = tag.attrs().next("name").value();

    // copyright?
    let copyright = if let Some((_, content)) = ready!(parser.next_tag_if("copyright")) {
        let (tag, _) = ready!(parser.next_tag("copyright"));
        assert!(tag.is_closing());
        Some(Bytes::new(content.trim_ascii()))
    } else {
        None
    };

    // description?
    let description = ready!(parse_description(&mut parser));

    element::Protocol {
        name,
        copyright,
        description,
    }.generate_header(output);

    advance_buffer!(parser, bytes, buffer);
    Ready(())
}

fn parse_description(parser: &mut Parser) -> Poll<Option<element::Description>> {
    // <!ELEMENT description (#PCDATA)>
    //   <!ATTLIST description summary CDATA #REQUIRED>
    if let Some((tag, content)) = ready!(parser.next_tag_if("description")) {
        let is_self_close = tag.is_self_close();
        let summary = tag.attrs().next("summary").value();
        // some description is self closing tag
        if !is_self_close {
            let (tag, _) = ready!(parser.next_tag("description"));
            assert!(tag.is_closing());
        }
        Ready(Some(element::Description { summary, content }))
    } else {
        Ready(None)
    }
}

fn parse_interface(buffer: &mut FileBuffer, output: &mut impl Write) -> Poll<bool> {
    let bytes = buffer.as_bytes();
    let mut parser = Parser::new(bytes);

    // <!ELEMENT interface (description?,(request|event|enum)+)>
    //   <!ATTLIST interface name CDATA #REQUIRED>
    //   <!ATTLIST interface version CDATA #REQUIRED>
    //   <!ATTLIST interface frozen CDATA #IMPLIED>
    let Some((tag, _)) = ready!(parser.next_tag_if("interface")) else {
        return Ready(false);
    };
    let mut attrs = tag.attrs();
    let name = attrs.next("name").value();
    let version = atou(attrs.next("version").value_slice());
    let frozen = match attrs.next_if("frozen") {
        Some(attr) => {
            assert_eq!(attr.value_slice(), b"true");
            true
        }
        None => false,
    };

    let description = if let Some((tag, content)) = ready!(parser.next_tag_if("description")) {
        let summary = tag.attrs().next("summary").value();
        let (tag, _) = ready!(parser.next_tag("description"));
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
    advance_buffer!(parser, bytes, buffer);

    // ===== request/event =====

    let mut state = InterfaceOpCode::new();
    loop {
        let Ready(ok) = parse_operation(&mut state, buffer, output) else {
            buffer.read();
            continue;
        };
        if !ok {
            break;
        }
    }

    // ===== end request/event =====

    loop {
        let bytes = buffer.as_bytes();
        let mut parser = Parser::new(bytes);
        let Ready((tag, _)) = parser.next_tag("interface") else {
            buffer.read();
            continue;
        };
        assert!(tag.is_closing());
        advance_buffer!(parser, bytes, buffer);
        break;
    }
    element::Interface::generate_trailer(output);

    Ready(true)
}

struct InterfaceOpCode {
    request: u32,
    event: u32,
}

impl InterfaceOpCode {
    fn new() -> Self {
        Self { request: 0, event: 0 }
    }

    fn request(&mut self) -> u32 {
        let r = self.request;
        self.request += 1;
        r
    }

    fn event(&mut self) -> u32 {
        let e = self.event;
        self.event += 1;
        e
    }
}

fn parse_operation(
    state: &mut InterfaceOpCode,
    buffer: &mut FileBuffer,
    output: &mut impl Write,
) -> Poll<bool> {
    if let Some(ok) = ready!(parse_enum(buffer, output)) {
        return Ready(ok)
    }

    let bytes = buffer.as_bytes();
    let mut parser = Parser::new(bytes);

    // <!ELEMENT request (description?,arg*)>
    // <!ELEMENT event (description?,arg*)>
    //   <!ATTLIST _ name CDATA #REQUIRED>
    //   <!ATTLIST _ type CDATA #IMPLIED>
    //   <!ATTLIST _ since CDATA #IMPLIED>
    //   <!ATTLIST _ deprecated-since CDATA #IMPLIED>
    let Some((tag, _)) = ready!(parser.next_tag_if_in(&["request", "event"])) else {
        return Ready(false);
    };

    let (kind, opcode) = match tag.name_slice() {
        b"request" => (element::OpKind::Request, state.request()),
        b"event" => (element::OpKind::Event, state.event()),
        _ => unreachable!(),
    };
    let mut attrs = tag.attrs();
    let name = attrs.next("name").value();
    let mut destructor = false;
    let mut since = None;
    let mut deprecated_since = None;

    while let Some(attr) = attrs.next_if_in(&["type", "since", "deprecated-since"]) {
        match attr.name_slice() {
            b"type" => {
                assert_eq!(attr.value_slice(), b"destructor");
                destructor = true;
            }
            b"since" => {
                since = Some(atou(attr.value_slice()));
            }
            b"deprecated-since" => {
                deprecated_since = Some(atou(attr.value_slice()));
            }
            _ => unreachable!(),
        }
    }

    let description = ready!(parse_description(&mut parser));
    let mut args = vec![];

    while let Some((tag, _)) = ready!(parser.next_tag_if("arg")) {
        // <!ELEMENT arg (description?)>
        //   <!ATTLIST arg name CDATA #REQUIRED>
        //   <!ATTLIST arg type CDATA #REQUIRED>
        //   <!ATTLIST arg summary CDATA #IMPLIED>
        //   <!ATTLIST arg interface CDATA #IMPLIED>
        //   <!ATTLIST arg allow-null CDATA #IMPLIED>
        //   <!ATTLIST arg enum CDATA #IMPLIED>
        let mut attrs = tag.attrs();
        let name = attrs.next("name").value();
        let ty = element::Type::from_wl_type(attrs.next("type").value_slice());
        let mut summary = None;
        let mut interface = None;
        let mut allow_null = false;
        let mut enum_name = None;

        while let Some(attr) = attrs.next_if_in(&["summary", "interface", "allow-null", "enum"]) {
            match attr.name_slice() {
                b"summary" => {
                    summary = Some(attr.value());
                }
                b"interface" => {
                    interface = Some(attr.value());
                }
                b"enum" => {
                    enum_name = Some(attr.value());
                }
                b"allow-null" => {
                    allow_null = match attr.value_slice() {
                        b"true" => true,
                        b"false" => false,
                        _ => unreachable!(),
                    };
                }
                _ => unreachable!(),
            }
        }
        let description = ready!(parse_description(&mut parser));
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

    let (tag, _) = ready!(parser.next_tag(kind.as_str()));
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
    advance_buffer!(parser, bytes, buffer);
    Ready(true)
}

fn parse_enum(buffer: &mut FileBuffer, output: &mut impl Write) -> Poll<Option<bool>> {
    let bytes = buffer.as_bytes();
    let mut parser = Parser::new(bytes);

    // <!ELEMENT enum (description?,entry*)>
    //   <!ATTLIST enum name CDATA #REQUIRED>
    //   <!ATTLIST enum since CDATA #IMPLIED>
    //   <!ATTLIST enum bitfield CDATA #IMPLIED>
    let Some((tag, _)) = ready!(parser.next_tag_if("enum")) else {
        return Ready(None);
    };

    let mut attrs = tag.attrs();
    let name = attrs.next("name").value();
    let mut since = None;
    let mut bitfield = false;

    while let Some(attr) = attrs.next_if_in(&["since", "bitfield"]) {
        match attr.name_slice() {
            b"since" => {
                since = Some(atou(attr.value_slice()));
            }
            b"bitfield" => {
                bitfield = match attr.value_slice() {
                    b"true" => true,
                    b"false" => false,
                    _ => unreachable!()
                };
            }
            _ => unreachable!(),
        }
    }

    let description = ready!(parse_description(&mut parser));
    let mut entries = vec![];

    while let Some((tag, _)) = ready!(parser.next_tag_if("entry")) {
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

        while let Some(attr) = attrs.next_if_in(&["summary", "interface", "allow-null", "enum"]) {
            match attr.name_slice() {
                b"summary" => {
                    summary = Some(attr.value());
                }
                b"since" => {
                    since = Some(atou(attr.value_slice()));
                }
                b"deprecated-since" => {
                    deprecated_since = Some(atou(attr.value_slice()));
                }
                _ => unreachable!(),
            }
        }

        let description = ready!(parse_description(&mut parser));
        entries.push(element::Entry {
            name,
            value,
            summary,
            description,
            since,
            deprecated_since,
        });
    }

    let (tag, _) = ready!(parser.next_tag("enum"));
    assert!(tag.is_closing());

    element::Enum {
        name,
        description,
        since,
        bitfield,
        entries,
    }.generate(output);
    advance_buffer!(parser, bytes, buffer);
    Ready(Some(true))
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

#[derive(Default)]
struct Bytes {
    inner: &'static [u8],
}

impl Bytes {
    fn new(bytes: &[u8]) -> Self {
        // SAFETY: lmao deez nutz
        let inner = unsafe { std::mem::transmute::<&[u8], &[u8]>(bytes) };
        Self { inner }
    }

    fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.inner) }
    }

    fn to_camel_case(&self) -> Box<str> {
        let mut string = String::with_capacity(self.len());
        let mut chars = self.as_str().chars();

        let prefix = chars.next().expect("name should be non-empty");

        // some enum variant starts with digit
        if prefix.is_ascii_digit() {
            string.push('_');
        }

        string.push(prefix.to_ascii_uppercase());

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

            string.push(ch);
        }

        string.into_boxed_str()
    }
}

impl std::fmt::Display for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::fmt::Debug for Bytes {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

impl std::ops::Deref for Bytes {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}
