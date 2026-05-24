use std::collections::HashMap;

use crate::error::{Error, ErrorExt, err, span};
use crate::schema::*;
use crate::str::Str;

// trait Parse: Sized {
//     fn parse(parser: &mut Parser) -> Result<Self, Error>;
// }

pub fn parse_wayland(string: Str) -> Result<Protocol, Error> {
    let mut parser = Parser { string };
    let prolog = parser.next_tag_string()?;
    if &*prolog != "<?xml version=\"1.0\" encoding=\"UTF-8\"?>" {
        return err!("unexpected xml prolog: `{prolog}`");
    }
    parse_protocol(&mut parser)
}

fn parse_protocol(parser: &mut Parser) -> Result<Protocol, Error> {
    let attrs = parser.assert_open_tag("protocol").and_then(parse_attr)?;
    Ok(Protocol {
        name: attrs.get_attr("name")?.clone(),
        copyright: if parser.peek_tag_name()? == "copyright" {
            parser.assert_open_tag("copyright")?;
            let cp = parser.next_plain()?;
            parser.assert_close_tag("copyright")?;
            Some(cp)
        } else {
            None
        },
        desc: peek_description(parser)?,
        interfaces: {
            let mut interfaces = Vec::with_capacity(8);
            while !parser.is_close_tag("protocol") {
                let _span = span("<interface>");
                interfaces.push(parse_interface(parser)?);
            }
            parser.assert_close_tag("protocol")?;
            interfaces
        },
    })
}

fn parse_interface(parser: &mut Parser) -> Result<Interface, Error> {
    let attrs = parser.assert_open_tag("interface").and_then(parse_attr)?;
    Ok(Interface {
        name: attrs.get_attr("name")?.clone(),
        version: attrs.get_attr("version")?.parse().cx("invalid version")?,
        frozen: attrs
            .get(&"frozen".into())
            .map(|e| e.parse().cx("invalid frozen value"))
            .transpose()?,
        desc: peek_description(parser)?,
        items: {
            let mut items = Vec::with_capacity(4);

            while !parser.is_close_tag("interface") {
                let _span = span("<item>");
                let item = match parser.peek_tag_name()? {
                    "request" | "event" => Item::Operation(parse_operation(parser)?),
                    "enum" => Item::Enum(parse_enum(parser)?),
                    tag => return err!("unexpected `<{tag}>`"),
                };
                items.push(item);
            }

            parser.assert_close_tag("interface")?;
            items
        },
    })
}

fn parse_operation(parser: &mut Parser) -> Result<Operation, Error> {
    let (tag_name, attrs, self_close) = parser.next_tag()?;
    if self_close {
        return err!("unexpected `<{tag_name}>` as self closing tag");
    }
    let attrs = parse_attr(attrs)?;
    Ok(Operation {
        kind: match &tag_name == "request" {
            true => OpKind::Request,
            false => OpKind::Event,
        },
        name: attrs.get_attr("name")?.clone(),
        ty: attrs.get(&"type".into()).cloned(),
        since: parse_optional_int(&attrs, "since")?,
        dep_since: parse_optional_int(&attrs, "deprecated-since")?,
        desc: peek_description(parser)?,
        args: {
            let mut args = Vec::with_capacity(4);

            while !parser.is_close_tag(&tag_name) {
                let _span = span("<arg>");
                args.push(parse_arg(parser)?);
            }

            parser.assert_close_tag(&tag_name)?;
            args
        },
    })
}

fn parse_arg(parser: &mut Parser) -> Result<Arg, Error> {
    let attrs = parser.assert_self_closing_tag("arg").and_then(parse_attr)?;
    Ok(Arg {
        name: attrs.get_attr("name")?.clone(),
        ty: attrs.get_attr("type")?.clone(),
        summary: attrs.get(&"summary".into()).cloned(),
        interface: attrs.get(&"interface".into()).cloned(),
        allow_null: attrs
            .get(&"allow_null".into())
            .map(|e| e.parse().cx("invalid `allow-null` value"))
            .transpose()?,
        enum_: attrs.get(&"enum".into()).cloned(),
        desc: peek_description(parser)?,
    })
}

fn parse_enum(parser: &mut Parser) -> Result<Enum, Error> {
    let attrs = parser.assert_open_tag("enum").and_then(parse_attr)?;
    Ok(Enum {
        name: attrs.get_attr("name")?.clone(),
        since: parse_optional_int(&attrs, "since")?,
        bitfield: attrs
            .get(&"version".into())
            .map(|e| e.parse().cx("invalid frozen value"))
            .transpose()?,
        desc: peek_description(parser)?,
        entries: {
            let mut entries = Vec::with_capacity(4);
            while !parser.is_close_tag("enum") {
                let _span = span("<entries>");
                entries.push(parse_entry(parser)?);
            }
            parser.assert_close_tag("enum")?;
            entries
        },
    })
}

fn parse_entry(parser: &mut Parser) -> Result<Entry, Error> {
    let attrs = parser.assert_self_closing_tag("entry").and_then(parse_attr)?;
    Ok(Entry {
        name: attrs.get_attr("name")?.clone(),
        value: attrs.get_attr("name")?.clone(),
        summary: attrs.get(&"summary".into()).cloned(),
        since: parse_optional_int(&attrs, "since")?,
        dep_since: parse_optional_int(&attrs, "deprecated-since")?,
        desc: peek_description(parser)?,
    })
}

fn peek_description(parser: &mut Parser) -> Result<Option<Description>, Error> {
    if parser.peek_tag_name()? != "description" {
        return Ok(None);
    }

    let (_, attrs, self_closing) = parser.next_tag()?;
    let attrs = parse_attr(attrs)?;
    let summary = attrs.get_attr("summary")?.clone();

    // there is self closing description
    let content = if self_closing {
        Str::from_static("")
    } else {
        parser.next_plain()?
    };

    if !self_closing {
        parser.assert_close_tag("description")?;
    }

    Ok(Some(Description {
        summary,
        content,
    }))
}

fn parse_optional_int(
    attrs: &HashMap<Str, Str>,
    cx: &'static str,
) -> Result<Option<std::num::NonZeroU32>, Error> {
    attrs
        .get(&cx.into())
        .map(|value| match value.parse() {
            Ok(ok) => Ok(ok),
            Err(_) => err!("invalid {cx} value: `{value}`"),
        })
        .transpose()
}

// ===== xml parser =====

fn parse_tag(string: Str) -> Result<(Str, Str, bool), Error> {
    let len = string.find([' ', '>']).cx("expected space or `>`")?;
    let self_close = &string[string.len() - 2..] == "/>";
    let name = string.slice(1..len);
    let attrs = string.slice(len..);
    Ok((name, attrs, self_close))
}

fn parse_attr(mut string: Str) -> Result<HashMap<Str, Str>, Error> {
    std::iter::from_fn(|| {
        let name_len = string.find('=')?;
        let key = string.split_to(name_len).trim_start();
        if &string[..2] != "=\"" {
            return Some(err!("bad attribute separator for `{key}`"));
        }
        string.advance(2);
        let Some(len) = string.find('"') else {
            return Some(err!("no closing value for `{key}`"));
        };
        let val = string.split_to(len);
        string.advance(1);
        Some(Ok((key, val)))
    })
    .collect()
}

struct Parser {
    string: Str,
}

impl Parser {
    fn skip_plain(&mut self) -> Result<(), Error> {
        loop {
            let adv = self.string.find('<').cx("expected `<`")?;
            self.string.advance(adv);
            if !self.string.starts_with("<!--") {
                break;
            }
            self.string.advance(1);
        }
        Ok(())
    }

    fn next_tag_string(&mut self) -> Result<Str, Error> {
        self.skip_plain()?;
        let end_idx = self.string.find('>').cx("expected `>`")?;
        let len = end_idx + 1;
        Ok(self.string.split_to(len))
    }

    fn next_tag(&mut self) -> Result<(Str, Str, bool), Error> {
        self.next_tag_string().and_then(parse_tag)
    }

    fn next_plain(&mut self) -> Result<Str, Error> {
        let len = self.string.find('<').cx("expected `<`")?;
        Ok(self.string.split_to(len))
    }

    fn peek_tag_name(&self) -> Result<&str, Error> {
        let mut string = &*self.string;
        loop {
            let offset = string.find('<').cx("expected `<`")?;
            string = &string[offset..];
            if !string.starts_with("<!--") {
                break;
            }
            string = &string[1..];
        }
        let len = string.find([' ', '>']).cx("expected `>`")?;
        Ok(&string[1..len])
    }

    fn is_close_tag(&self, expected: &str) -> bool {
        fn inner(string: &str, expected: &str) -> Option<()> {
            let offset = string.find('<')?;
            let name = string[offset + 1..].strip_prefix('/')?;
            name.starts_with(expected).then_some(())
        }
        inner(&self.string, expected).is_some()
    }

    fn assert_open_tag(&mut self, expected: &str) -> Result<Str, Error> {
        let (name, attrs, self_closing) = self.next_tag()?;
        if &*name != expected {
            return err!("expected `<{expected}>` opening tag, found `<{name}>`");
        }
        if self_closing {
            return err!("unexpected `<{expected}>` as self closing tag");
        }
        Ok(attrs)
    }

    fn assert_close_tag(&mut self, expected: &str) -> Result<(), Error> {
        let tag_string = self.next_tag_string()?;
        let (tag, rest) = tag_string.split_at(2);
        if tag != "</" {
            return err!("expected `<{expected}>` as closing tag");
        }
        let (name, _) = rest.split_at(rest.len() - 1);
        if name != expected {
            return err!("expected `<{expected}>` closing tag, found `<{name}>`");
        }
        Ok(())
    }

    fn assert_self_closing_tag(&mut self, expected: &str) -> Result<Str, Error> {
        let (name, attrs, self_closing) = self.next_tag()?;
        if &*name != expected {
            return err!("expected `<{expected}>` opening tag, found `<{name}>`");
        }
        if !self_closing {
            return err!("expected `<{expected}>` as self closing tag");
        }
        Ok(attrs)
    }
}

trait MapExt {
    fn get_attr(&self, cx: &'static str) -> Result<Str, Error>;
}

impl MapExt for HashMap<Str, Str> {
    fn get_attr(&self, cx: &'static str) -> Result<Str, Error> {
        match self.get(&cx.into()) {
            Some(ok) => Ok(ok.clone()),
            None => err!("no `{cx}`")
        }
    }
}
