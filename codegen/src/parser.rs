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
        name: attrs.assert_attr("name")?,
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
            while !parser.try_close_tag("protocol") {
                let _span = span("<interface>");
                interfaces.push(parse_interface(parser)?);
            }
            interfaces
        },
    })
}

fn parse_interface(parser: &mut Parser) -> Result<Interface, Error> {
    let attrs = parser.assert_open_tag("interface").and_then(parse_attr)?;
    Ok(Interface {
        name: attrs.assert_attr("name")?,
        version: attrs.assert_attr("version")?.parse().cx("invalid version")?,
        frozen: attrs.get_parsed("frozen")?,
        desc: peek_description(parser)?,
        items: {
            let mut items = Vec::with_capacity(4);
            while !parser.try_close_tag("interface") {
                let _span = span("<item>");
                let item = match parser.peek_tag_name()? {
                    "request" | "event" => Item::Operation(parse_operation(parser)?),
                    "enum" => Item::Enum(parse_enum(parser)?),
                    tag => return err!("unexpected `<{tag}>`"),
                };
                items.push(item);
            }
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
        name: attrs.assert_attr("name")?,
        ty: attrs.get(&"type".into()).cloned(),
        since: attrs.get_parsed("since")?,
        dep_since: attrs.get_parsed("deprecated-since")?,
        desc: peek_description(parser)?,
        args: {
            let mut args = Vec::with_capacity(4);
            while !parser.try_close_tag(&tag_name) {
                let _span = span("<arg>");
                args.push(parse_arg(parser)?);
            }
            args
        },
    })
}

fn parse_arg(parser: &mut Parser) -> Result<Arg, Error> {
    let attrs = parser.assert_self_closing_tag("arg").and_then(parse_attr)?;
    Ok(Arg {
        name: attrs.assert_attr("name")?,
        ty: attrs.assert_attr("type")?,
        summary: attrs.get(&"summary".into()).cloned(),
        interface: attrs.get(&"interface".into()).cloned(),
        allow_null: attrs.get_parsed("allow-null")?,
        enum_: attrs.get(&"enum".into()).cloned(),
        desc: peek_description(parser)?,
    })
}

fn parse_enum(parser: &mut Parser) -> Result<Enum, Error> {
    let attrs = parser.assert_open_tag("enum").and_then(parse_attr)?;
    Ok(Enum {
        name: attrs.assert_attr("name")?,
        since: attrs.get_parsed("since")?,
        bitfield: attrs.get_parsed("bitfield")?,
        desc: peek_description(parser)?,
        entries: {
            let mut entries = Vec::with_capacity(4);
            while !parser.try_close_tag("enum") {
                let _span = span("<entries>");
                entries.push(parse_entry(parser)?);
                parser.try_close_tag("entry");
            }
            entries
        },
    })
}

fn parse_entry(parser: &mut Parser) -> Result<Entry, Error> {
    let (attrs, _) = parser.assert_tag("entry")?;
    let attrs = parse_attr(attrs)?;
    Ok(Entry {
        name: attrs.assert_attr("name")?,
        value: attrs.assert_attr("name")?,
        summary: attrs.get(&"summary".into()).cloned(),
        since: attrs.get_parsed("since")?,
        dep_since: attrs.get_parsed("deprecated-since")?,
        desc: peek_description(parser)?,
    })
}

fn peek_description(parser: &mut Parser) -> Result<Option<Description>, Error> {
    if parser.peek_tag_name()? != "description" {
        return Ok(None);
    }
    let (_, attrs, self_closing) = parser.next_tag()?;
    let attrs = parse_attr(attrs)?;
    Ok(Some(Description {
        summary: attrs.assert_attr("summary")?,
        content: {
            let content = if self_closing {
                Str::from_static("")
            } else {
                parser.next_plain()?
            };
            if !self_closing {
                parser.assert_close_tag("description")?;
            }
            content
        },
    }))
}

// ===== xml parser =====

fn parse_tag(string: Str) -> Result<(Str, Str, bool), Error> {
    let len = string.find([' ', '>']).cx("expected space or `>`")?;
    let self_close = string.ends_with("/>");
    let name = string.slice(1..len);
    let attrs = string.slice(len..);
    Ok((name, attrs, self_close))
}

fn parse_attr(mut string: Str) -> Result<HashMap<Str, Str>, Error> {
    std::iter::from_fn(|| {
        let name_len = string.find('=')?;
        let key = string.split_to(name_len).trim_start();
        if !string.starts_with("=\"")  {
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

    fn try_close_tag(&mut self, expected: &str) -> bool {
        fn inner(string: &str, expected: &str) -> Option<usize> {
            let offset = string.find('<')?;
            let name = string.get(offset + 1..)?.strip_prefix('/')?;
            let suffix = name.strip_prefix(expected)?;
            suffix.starts_with('>').then_some(offset + 2 + expected.len() + 1)
        }
        match inner(&self.string, expected) {
            Some(cnt) => {
                self.string.advance(cnt);
                true
            },
            None => false,
        }
    }

    fn assert_tag(&mut self, expected: &str) -> Result<(Str, bool), Error> {
        let (name, attrs, self_closing) = self.next_tag()?;
        if &*name == expected {
            Ok((attrs, self_closing))
        } else {
            err!("expected `<{expected}>` opening tag, found `<{name}>`")
        }
    }

    fn assert_open_tag(&mut self, expected: &str) -> Result<Str, Error> {
        match self.assert_tag(expected)? {
            (attrs, false) => Ok(attrs),
            _ => err!("unexpected `<{expected}>` as self closing tag"),
        }
    }

    fn assert_self_closing_tag(&mut self, expected: &str) -> Result<Str, Error> {
        match self.assert_tag(expected)? {
            (attrs, true) => Ok(attrs),
            _ => err!("expected `<{expected}>` as self closing tag"),
        }
    }

    fn assert_close_tag(&mut self, expected: &str) -> Result<(), Error> {
        let tag_string = self.next_tag_string()?;
        let (tag, rest) = tag_string.split_at(2);
        if tag != "</" {
            return err!("expected `<{expected}>` as closing tag");
        }
        let (name, _) = rest.split_at(rest.len() - 1);
        if name == expected {
            Ok(())
        } else {
            err!("expected `<{expected}>` closing tag, found `<{name}>`")
        }
    }
}

trait MapExt {
    fn assert_attr(&self, cx: &'static str) -> Result<Str, Error>;

    fn get_parsed<P>(&self, cx: &'static str) -> Result<Option<P>, Error>
    where
        P: std::str::FromStr,
        P::Err: std::fmt::Display;
}

impl MapExt for HashMap<Str, Str> {
    fn assert_attr(&self, cx: &'static str) -> Result<Str, Error> {
        match self.get(&cx.into()) {
            Some(ok) => Ok(ok.clone()),
            None => err!("expected `{cx}` attribute"),
        }
    }

    fn get_parsed<P>(&self, cx: &'static str) -> Result<Option<P>, Error>
    where
        P: std::str::FromStr,
        P::Err: std::fmt::Display,
    {
        self.get(&cx.into())
            .map(|value| {
                value
                    .parse()
                    .map_err(|e| Error::new(format!("invalid {cx} value `{value}`: {e}")))
            })
            .transpose()
    }
}
