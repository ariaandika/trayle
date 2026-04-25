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

    let mut parser = Parser::new(File::open(path).unwrap());
    let mut output = std::io::stdout().lock();

    // <!ELEMENT protocol (copyright?, description?, interface+)>
    let tag = parser.next_tag_assert("protocol");
    let name = tag.attrs().next_assert("name");
    writeln!(output, "//! {}", f(name));
    writeln!(output, "pub use super::{{Array, Type, Encode}};");
    writeln!(output, "pub use std::os::fd::RawFd;");
    writeln!(output, "pub use std::num::NonZeroU32;");

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
    let name = SmallBuf::new(attrs.next_assert("name"));
    let version = atou(attrs.next_assert("version"));

    // ===== description? =====

    writeln!(output);
    process_description(parser, output, "");

    writeln!(output, "pub mod {name} {{");
    writeln!(output, "    use super::*;");
    writeln!(output, "    /// {name} version");
    writeln!(output, "    pub const VERSION: u32 = {version};");

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
                process_operation(OpKind::Request, reqcode, parser, output);
                reqcode += 1;
            }
            b"event" => {
                process_operation(OpKind::Event, evcode, parser, output);
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

    if !summary.is_empty() {
        writeln!(output, "{pad}/// {}", f(summary));
        writeln!(output, "{pad}///");
    }

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

fn process_operation<O: Write>(op: OpKind, opcode: usize, parser: &mut Parser, output: &mut O) {
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

    let tag = parser.next_tag_assert(op.as_str());
    let mut attrs = tag.attrs();
    let name = attrs.next_assert("name");
    let name = if name == b"move" {
        // some request is a rust keyword
        SmallBuf::new(b"r#move")
    } else {
        SmallBuf::new(name)
    };

    let mut is_type_destructor = false;
    let mut since = None;
    let mut dep_since = None;

    while let Some(attr) = attrs.try_next() {
        match attr.name() {
            b"type" => {
                // `type` can only contains literal `destructor`
                assert_eq!(attr.value(), b"destructor");
                is_type_destructor = true;
            }
            b"since" => since = Some(atou(attr.value())),
            b"deprecated-since" => dep_since = Some(atou(attr.value())),
            _ => unreachable!()
        }
    }

    process_description(parser, output, "    ");
    if let Some(since) = since {
        writeln!(output, "    ///");
        writeln!(output, "    /// since: {since}");
    }
    if let Some(dep_since) = dep_since {
        writeln!(output, "    ///");
        writeln!(output, "    /// deprecated-since: {dep_since}");
    }

    // ===== mod =====
    writeln!(output, "    pub mod {} {{", f(&name));
    writeln!(output, "        use super::*;");
    writeln!(output, "        pub const OPCODE: u32 = {opcode};");
    writeln!(output, "        pub const IS_REQUEST: bool = {};", op.is_request());
    writeln!(output, "        pub const IS_EVENT: bool = {};", op.is_event());
    writeln!(output, "        pub const IS_TYPE_DESTRUCTOR: bool = {is_type_destructor};");

    const PAD: &str = "        ";
    const PA2: &str = "            ";
    const PA3: &str = "                ";
    const PA4: &str = "                    ";

    let mut args = Vec::with_capacity(8);
    let mut lifetime = "";
    while parser.peek_tag().name() == b"arg" {
        let arg = Arg::parse(parser);
        if matches!(arg.ty, Type::String | Type::Array) {
            lifetime = "<'a>";
        }
        args.push(arg);
    }

    fn ty_name(arg: &Arg) -> &'static str {
        match arg.ty {
            Type::Int => "i32",
            Type::Uint => "u32",
            Type::Fixed => "f32",
            Type::String => "&'a str",
            Type::Array => "&'a Array",
            Type::Fd => "RawFd",
            Type::NewId => "NonZeroU32",
            Type::Object => "NonZeroU32",
        }
    }

    // ===== struct Encoded =====
    writeln!(output);
    writeln!(output, "{PAD}pub struct Encoded{lifetime} {{");
    for arg in &args {
        // name type $(summary interface allow-null enum)?
        if let Some(sum) = arg.summary.as_ref() {
            writeln!(output, "{PA2}/// {}", f(sum));
        }
        let ty_name = if arg.allow_null {
            format_args!("Option<{}>", ty_name(arg))
        } else {
            format_args!("{}", ty_name(arg))
        };
        writeln!(output, "{PA2}pub {}: {},", f(&arg.name), ty_name);
    }
    writeln!(output, "{PAD}}}");

    // ===== impl Encoded =====
    writeln!(output);
    writeln!(output, "{PAD}impl{lifetime} Encoded{lifetime} {{");

    // ===== fn size() =====
    writeln!(output, "{PA2}pub fn size(&self) -> usize {{");
    write!(output, "{PA3}");
    let mut is_first = true;
    for arg in &args {
        if is_first {
            is_first = false;
        } else {
            write!(output, " + ");
        }
        write!(output, "Type::size(&self.{})", f(&arg.name));
    }
    writeln!(output);
    writeln!(output, "{PA2}}}");

    // ===== fn encode() =====
    writeln!(output);
    writeln!(output, "{PA2}pub fn encode(&self, buf: &mut [u8]) {{");
    if let [arg] = args.as_slice() {
        writeln!(output, "{PA3}if buf.len() != Type::size(&self.{}) {{", f(&arg.name));
        writeln!(output, "{PA3}    panic!(\"encoding failed, buffer is too small\");");
        writeln!(output, "{PA3}}}");
        writeln!(output, "{PA3}unsafe {{ Encode::encode_unchecked(&self.{}, buf) }};", f(&arg.name));
    } else {
        writeln!(output, "{PA3}if buf.len() != self.size() {{");
        writeln!(output, "{PA3}    panic!(\"encoding failed, buffer is too small\");");
        writeln!(output, "{PA3}}}");
        writeln!(output, "{PA3}unsafe {{");
        for arg in &args {
            writeln!(output, "{PA4}let (write, buf) = buf.split_at_mut_unchecked(Type::size(&self.{}));", f(&arg.name));
            writeln!(output, "{PA4}Encode::encode_unchecked(&self.{}, write);", f(&arg.name));
        }
        writeln!(output, "{PA4}let _ = buf;");
        writeln!(output, "{PA3}}}");
    }
    writeln!(output, "{PA2}}}");

    // ===== end impl Encoded =====
    writeln!(output, "{PAD}}}");

    // ===== end mod =====
    writeln!(output, "    }}");

    parser.next_closing_tag_assert(op.as_str());
}

fn process_enum<O: Write>(parser: &mut Parser, output: &mut O) {
    // <!ELEMENT enum (description?,entry*)>
    //   <!ATTLIST enum name CDATA #REQUIRED>
    //   <!ATTLIST enum since CDATA #IMPLIED>
    //   <!ATTLIST enum bitfield CDATA #IMPLIED>

    let tag = parser.next_tag_assert("enum");
    let mut attrs = tag.attrs();
    let name = SmallBuf::new_camel_case(attrs.next_assert("name"));

    let mut since = None;
    let mut is_bitfield = false;

    while let Some(attr) = attrs.try_next() {
        let atr_name = attr.name();
        match atr_name {
            b"since" => since = Some(atou(attr.value())),
            b"bitfield" => {
                is_bitfield = match attr.value() {
                    b"true" => true,
                    b"false" => false,
                    _ => unreachable!(),
                }
            }
            _ => unreachable!(),
        }
    }

    // some enum does not have description
    process_description(parser, output, "    ");
    if let Some(since) = since {
        writeln!(output, "    ///");
        writeln!(output, "    /// since: {since}");
    }
    writeln!(output, "    #[derive(Debug)]");
    writeln!(output, "    pub enum {name} {{");
    while parser.peek_tag().name() == b"entry" {
        process_entry(parser, output);
    }
    writeln!(output, "    }}");
    writeln!(output);
    writeln!(output, "    impl {name} {{");
    writeln!(
        output,
        "        pub const IS_BITFIELD: bool = {is_bitfield};"
    );
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
    let is_self_close = tag.is_self_close();

    let mut attrs = tag.attrs();
    let name = SmallBuf::new_camel_case(attrs.next_assert("name"));
    let value = SmallBuf::new(attrs.next_assert("value"));

    let mut entry_summary = false;
    let mut since = None;
    let mut dep_since = None;

    while let Some(attr) = attrs.try_next() {
        let value = attr.value();
        match attr.name() {
            b"summary" => {
                entry_summary = true;
                write!(output, "{PAD}///");
                // there is summary that wrapped to a new line
                for line in f(value).split('\n') {
                    write!(output, " {line}");
                }
                writeln!(output);
            }
            b"since" => since = Some(atou(value)),
            b"deprecated-since" => dep_since = Some(atou(value)),
            _ => unreachable!(),
        }
    }

    if parser.peek_tag().name() == b"description" {
        if entry_summary {
            writeln!(output, "{PAD}///");
        }
        process_description(parser, output, PAD);
    }
    if let Some(since) = since {
        writeln!(output, "{PAD}///");
        writeln!(output, "{PAD}/// since: {since}");
    }
    if let Some(dep_since) = dep_since {
        writeln!(output, "{PAD}///");
        writeln!(output, "{PAD}/// deprecated-since: {dep_since}");
    }
    writeln!(output, "{PAD}{} = {value},", f(&name));

    if !is_self_close {
        parser.next_closing_tag_assert("entry");
    }
}

// ===== Arg =====

struct Arg {
    name: Vec<u8>,
    ty: Type,
    summary: Option<Vec<u8>>,
    /// If given, iface must be the name of some interface, and type of this argument must be either
    /// "object" or "new_id". This indicates that the existing or new object must have the interface
    /// iface. Use for other argument types is forbidden.
    ///
    /// If an interface from another protocol is used, then this creates a dependency between the
    /// protocols. If an application generates code for one protocol, then it must also generate
    /// code for all dependencies. Therefore this would not be a backwards compatible change.
    interface: Option<Vec<u8>>,
    /// Whether the argument value can be null on send. Defaults to "false", meaning it is illegal
    /// to send a null value. Can be used only when type is "string" or "object".
    allow_null: bool,
    /// If specified, indicates that the argument value should come from the enum named
    /// enum-cname-suffix. If the enumeration is a bitfield, then type must be "uint". Otherwise
    /// type must be either "uint" or "int".
    ///
    /// The name enum-cname-suffix refers to an enum in the same interface by default. If it is
    /// necessary to refer to an enumeration from another interface, the interface name can be given
    /// with a period:
    ///
    /// `enum`="`iface`.`enum-cname-suffix`"
    enum_: Option<Vec<u8>>,
}

impl Arg {
    fn parse(parser: &mut Parser) -> Self {
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
        let name = attrs.next_assert("name").to_vec();
        let ty = Type::from_bytes(attrs.next_assert("type"));
        let mut summary = None;
        let mut interface = None;
        let mut allow_null = false;
        let mut enum_ = None;

        while let Some(attr) = attrs.try_next() {
            let val = attr.value();
            match attr.name() {
                b"summary" => summary = Some(val.to_vec()),
                b"interface" => interface = Some(val.to_vec()),
                b"allow-null" => allow_null = match val {
                    b"true" => true,
                    b"false" => false,
                    _ => unreachable!("unknown allow-null value: {}", f(val)),
                },
                b"enum" => enum_ = Some(val.to_vec()),
                name => unknown_attribute(name),
            }
        }

        Self {
            name,
            ty,
            summary,
            interface,
            allow_null,
            enum_,
        }
    }
}

enum Type {
    /// 32-bit signed integer.
    Int,
    /// 32-bit unsigned integer.
    Uint,
    /// Signed 24.8-bit fixed-point value.
    Fixed,
    /// UTF-8 encoded string value, NUL byte terminated. Interior NUL bytes are not allowed.
    String,
    /// A byte array of arbitrary data.
    Array,
    /// A file descriptor.
    ///
    /// The file descriptor must be open and valid on send. It is not possible to pass a null value.
    Fd,
    /// Creates a new protocol object. A request or an event may have at most one new_id argument.
    ///
    /// If interface is specified, the new protocol object shall have the specified interface, and the new object’s (interface) version shall be the version of the object on which the request or event is being sent.
    ///
    /// If interface is not specified, the request shall implicitly have two additional arguments: A string for an interface name, and a uint for the new object’s version. Leaving the interface unspecified is reserved for special use, wl_registry.bind for example.
    ///
    /// Note
    ///
    /// An event argument must always specify the new_id interface.
    NewId,
    /// Reference to an existing protocol object.
    ///
    /// The attribute interface should be specified. Otherwise IPC libraries cannot enforce the interface, and checking the interface falls on user code and specification text.
    Object,
}

impl Type {
    fn from_bytes(ty: &[u8]) -> Self {
        match ty {
            b"int" => Self::Int,
            b"uint" => Self::Uint,
            b"fixed" => Self::Fixed,
            b"string" => Self::String,
            b"array" => Self::Array,
            b"fd" => Self::Fd,
            b"new_id" => Self::NewId,
            b"object" => Self::Object,
            _ => unreachable!("unknown type: {}", f(ty)),
        }
    }
}

// ===== Util =====

fn f(bytes: &[u8]) -> &str {
    // SAFETY: the parser guarantee that prolog is `encoding="UTF-8"`
    unsafe { str::from_utf8_unchecked(bytes) }
}

fn atou(bytes: &[u8]) -> u16 {
    bytes
        .iter()
        .fold(0u16, |acc, next| match next.wrapping_sub(b'0') {
            b @ 0..=9 => acc.wrapping_mul(10).wrapping_add(b as _),
            _ => panic!("non integer"),
        })
}

fn unknown_attribute(name: &[u8]) -> ! {
    panic!("unknown attribute: {:?}", str::from_utf8(name))
}

#[derive(Debug)]
enum OpKind {
    Request,
    Event,
}

impl OpKind {
    fn as_str(&self) -> &'static str {
        match self {
            OpKind::Request => "request",
            OpKind::Event => "event",
        }
    }

    /// Returns `true` if the op kind is [`Request`].
    ///
    /// [`Request`]: OpKind::Request
    fn is_request(&self) -> bool {
        matches!(self, Self::Request)
    }

    /// Returns `true` if the op kind is [`Event`].
    ///
    /// [`Event`]: OpKind::Event
    fn is_event(&self) -> bool {
        matches!(self, Self::Event)
    }
}

struct SmallBuf([u8; 32]);

impl SmallBuf {
    /// Panics if name is too long.
    fn new(name: &[u8]) -> Self {
        let mut buf = [0u8; _];
        buf[0] = name.len() as u8;
        let Some(b) = buf.get_mut(1..1 + name.len()) else {
            panic!("name too long: {}", f(name));
        };
        b.copy_from_slice(name);
        Self(buf)
    }

    /// Change identifier to camel case.
    ///
    /// This fix identifier that is rust keyword or digit only.
    ///
    /// Panics if name is too long.
    fn new_camel_case(name: &[u8]) -> Self {
        let mut buf = [0u8; _];
        let (len, b) = buf.split_first_mut().unwrap();

        let prefix = name.first().expect("empty identifier");
        let mut name_iter = name.iter();

        if prefix.is_ascii_digit() {
            b[0] = b'D';
        } else {
            name_iter.next();
            b[0] = prefix.to_ascii_uppercase();
        };
        *len += 1;

        while let Some(byte) = name_iter.next() {
            if *byte == b'_' {
                let Some(next) = name_iter.next() else {
                    break;
                };
                b[*len as usize] = next.to_ascii_uppercase();
            } else {
                b[*len as usize] = *byte;
            }
            *len += 1;
        }

        Self(buf)
    }
}

impl std::ops::Deref for SmallBuf {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0[1..1 + self.0[0] as usize]
    }
}

impl std::fmt::Display for SmallBuf {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f(self).fmt(fmt)
    }
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

