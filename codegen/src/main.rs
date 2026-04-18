use std::env::args;
use std::fs::File;
use std::io::Write;

use crate::parser::Parser;

mod parser;

fn main() {
    let Some(path) = args().nth(1) else {
        panic!("path arguments is required");
    };

    let mut parser = Parser::new(File::open(path).unwrap());

    let mut proto = parser.next_tag();
    assert!(proto.name().0.eq(b"protocol"));

    let mut stdout = std::io::stdout().lock();

    loop {
        let (name, is_closing) = parser.peek_tag();
        match (name.as_slice(), is_closing) {
            (b"copyright", false) => {
                parser.next_tag();
                parser.next_plain();
                parser.next_tag();
                continue
            }
            (b"protocol", true) => break,
            (b"interface", false) => {
                interface(&mut parser, &mut stdout);
            }
            (name, close) => panic!(
                "unexpected tag: {}{}",
                str::from_utf8(name).unwrap(),
                if close { "(closing)" } else { "" },
            )
        }
    }
}

fn interface<O: Write>(parser: &mut Parser, mut output: O) {
    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"interface"[..]);
    assert!(!is_close);

    let (key, name) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"name"[..]);
    let (key, version) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"version"[..]);

    // ===== description =====

    print_description(parser, &mut output);

    output.write_all(b"pub mod ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" {\n").unwrap();
    output.write_all(b"/// ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" version\npub const VERSION: u32 = ").unwrap();
    output.write_all(&version).unwrap();
    output.write_all(b";\n\n").unwrap();

    // ===== operations =====

    let mut reqcode = 0;
    let mut evcode = 0;

    loop {
        let (name, _) = parser.peek_tag();
        match name.as_slice() {
            b"request" => {
                let code = reqcode;
                reqcode += 1;
                print_request(parser, &mut output, code);
                output.write_all(b"\n").unwrap();
            }
            b"event" => {
                let code = evcode;
                evcode += 1;
                print_event(parser, &mut output, code);
                output.write_all(b"\n").unwrap();
            }
            b"enum" => {
                print_enum(parser, &mut output);
                output.write_all(b"\n").unwrap();
            }
            b"interface" => break,
            name => panic!("unknown interface property: {}", str::from_utf8(name).unwrap()),
        }
    }

    output.write_all(b"}\n\n").unwrap();

    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), b"interface");
    assert!(is_close);
}

fn print_request<O: Write>(parser: &mut Parser, output: O, opcode: usize) {
    print_operation("request", opcode, parser, output);
}

fn print_event<O: Write>(parser: &mut Parser, output: O, opcode: usize) {
    print_operation("event", opcode, parser, output);
}

fn print_operation<O: Write>(op: &str, opcode: usize, parser: &mut Parser, mut output: O) {
    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), op.as_bytes());
    assert!(!is_close);

    let (key, mut name) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"name"[..]);

    // some request is same as rust keyword
    if name == b"move" {
        name = b"r#move".to_vec();
    }

    print_description(parser, &mut output);

    output.write_all(b"        pub mod ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" {\n").unwrap();

    // ===== kind =====
    let mut cop = op.as_bytes().to_vec();
    cop[0].make_ascii_uppercase();
    output.write_all(b"pub const KIND: Kind = Kind::").unwrap();
    output.write_all(&cop).unwrap();
    output.write_all(b";\n").unwrap();

    // ===== opcode =====
    output.write_all(b"pub const OPCODE: u32 = ").unwrap();
    output.write_fmt(format_args!("{opcode}")).unwrap();
    output.write_all(b";\n\n").unwrap();

    // ===== write =====
    output.write_all(b"            pub fn write(").unwrap();

    loop {
        let (name, is_closing) = parser.peek_tag();
        match (name.as_slice(), is_closing) {
            (b"arg", _) => print_arg(parser, &mut output),
            (o, true) if o == op.as_bytes() => break,
            (name, _) => panic!("unexpected tag: {}",str::from_utf8(name).unwrap()),
        }
    }

    output.write_all(b") { todo!() }\n").unwrap();

    // ===== read =====
    // TODO:

    output.write_all(b"    }\n").unwrap();

    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), op.as_bytes());
    assert!(is_close);
}

fn print_description<O: Write>(parser: &mut Parser, mut output: O) {
    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"description"[..], "{:?}", str::from_utf8(&name).unwrap());
    assert!(!is_close);

    let (key, summary) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"summary"[..]);

    output.write_all(b"/// ").unwrap();
    output.write_all(&summary).unwrap();
    output.write_all(b"\n").unwrap();

    output.write_all(b"///\n").unwrap();

    let desc = parser.next_plain();

    for line in std::io::BufRead::lines(&desc[..]) {
        output.write_all(b"/// ").unwrap();
        output.write_all(line.unwrap().as_bytes().trim_ascii_start()).unwrap();
        output.write_all(b"\n").unwrap();
    }

    // there is self closed description
    if tag.is_self_close() {
        return
    }

    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"description"[..], "{:?}", str::from_utf8(name.as_slice()).unwrap());
    assert!(is_close);
}

fn print_enum<O: Write>(parser: &mut Parser, mut output: O) {
    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"enum"[..]);
    assert!(!is_close);

    let (key, name) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"name"[..]);

    // some enum does not have description
    if matches!(parser.peek_tag().0.as_slice(), b"description") {
        print_description(parser, &mut output);
    }

    output.write_all(b"pub enum ").unwrap();
    output.write_all(&name).unwrap();
    output.write_all(b" {\n").unwrap();

    loop {
        let (name, is_closing) = parser.peek_tag();
        match (name.as_slice(), is_closing) {
            (b"entry", _) => print_entry(parser, &mut output),
            (b"enum", true) => break,
            (b"!--", _) => {
                parser.next_tag();
            }
            (name, _) => panic!("unexpected tag: {}",str::from_utf8(name).unwrap()),
        }
    }

    output.write_all(b"}\n").unwrap();

    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"enum"[..]);
    assert!(is_close);
}

fn print_arg<O: Write>(parser: &mut Parser, mut output: O) {
    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"arg"[..]);
    assert!(!is_close);

    let (key, name) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"name"[..]);

    output.write_all(&name).unwrap();
    output.write_all(b": ").unwrap();

    let (key, ty) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"type"[..]);

    output.write_all(&ty).unwrap();

    let (key, val) = tag.next_attr().unwrap();

    if key == b"interface" {
        output.write_all(b"<").unwrap();
        output.write_all(&val).unwrap();
        output.write_all(b">").unwrap();
    }

    output.write_all(b",").unwrap();

    // TODO: ===== summary =====
}

fn print_entry<O: Write>(parser: &mut Parser, mut output: O) {
    let mut tag = parser.next_tag();
    let (name, is_close) = tag.name();
    assert_eq!(name.as_slice(), &b"entry"[..]);
    assert!(!is_close);

    let (key, mut name) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"name"[..]);

    // for some variant, it is same as rust keyword
    name[0].make_ascii_uppercase();

    // for some variant, it only contains digit
    if name.iter().all(|e|e.is_ascii_digit()) {
        name.insert(0, b'd');
    }

    let (key, value) = tag.next_attr().unwrap();
    assert_eq!(key.as_slice(), &b"value"[..]);

    // let (key, summary) = tag.next_attr().unwrap();
    if let Some((key, summary)) = tag.next_attr() {
        assert_eq!(key.as_slice(), &b"summary"[..]);

        output.write_all(b"/// ").unwrap();
        output.write_all(&summary).unwrap();
        output.write_all(b"\n").unwrap();
    }

    output.write_all(&name).unwrap();
    output.write_all(b" = ").unwrap();
    output.write_all(&value).unwrap();
    output.write_all(b",\n").unwrap();

    assert!(tag.is_self_close());
}

