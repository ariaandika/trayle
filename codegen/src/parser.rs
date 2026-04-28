use std::cmp;
use std::task::Poll::{self, *};
use std::task::ready;

use crate::Bytes;

macro_rules! some {
    ($e:expr) => {
        match $e {
            Some(ok) => ok,
            None => return Pending,
        }
    };
}

/// find whitespace or `'>'`.
fn tag_name(b: &u8) -> bool {
    b.is_ascii_whitespace() || *b == b'>'
}

/// find `'<'`.
fn tag_open(b: &u8) -> bool {
    *b == b'<'
}

/// find `'>'`.
fn tag_close(b: &u8) -> bool {
    *b == b'>'
}

/// find `'='`.
fn attr_name(b: &u8) -> bool {
    *b == b'='
}

// ===== Parser =====

pub struct Parser<'a> {
    buffer: &'a [u8],
}

impl<'a> std::fmt::Debug for Parser<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field(
                "buffer",
                &str::from_utf8(&self.buffer[..cmp::min(self.buffer.len(), 512)]),
            )
            .finish()
    }
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        assert_eq!(buffer.first(), Some(&b'<'));
        Self { buffer }
    }

    pub fn as_bytes(&self) -> &'a [u8] {
        self.buffer
    }
}

impl<'a> Parser<'a> {
    pub fn assert_prolog(&mut self, prolog: &str) -> Poll<()> {
        let (p, rest) = some!(self.buffer.split_at_checked(prolog.len()));
        assert_eq!(p, prolog.as_bytes());
        self.buffer = rest.trim_ascii_start();
        Ready(())
    }

    pub fn next_tag(&mut self, name: &str) -> Poll<(Tag, Bytes)> {
        match ready!(self.next_tag_inner(&[name])) {
            Ok(ok) => Ready(ok),
            Err(err) => panic!("expected `{name}`, found `{err}`"),
        }
    }

    pub fn next_tag_if(&mut self, name: &str) -> Poll<Option<(Tag, Bytes)>> {
        match ready!(self.next_tag_inner(&[name])) {
            Ok(ok) => Ready(Some(ok)),
            Err(_) => Ready(None),
        }
    }

    pub fn next_tag_if_in(&mut self, names: &[&str]) -> Poll<Option<(Tag, Bytes)>> {
        match ready!(self.next_tag_inner(names)) {
            Ok(ok) => Ready(Some(ok)),
            Err(_) => Ready(None),
        }
    }

    fn next_tag_inner(&mut self, expect_names: &[&str]) -> Poll<Result<(Tag, Bytes), Bytes>> {
        let Some((prefix, mut buf)) = self.buffer.split_first() else {
            unreachable!()
        };
        assert_eq!(*prefix as char, '<');

        let name_len = some!(buf.iter().position(tag_name));

        let is_closing = buf[0] == b'/';
        buf = &buf[is_closing as usize..];

        let name = &buf[..name_len - is_closing as usize];
        if !expect_names.iter().any(|ex|ex.as_bytes() == name) {
            return Ready(Err(Bytes::new(name)));
        }

        let tag_len = some!(buf.iter().position(tag_close)) + 1;
        let (tag_buf, buf) = buf.split_at(tag_len);
        let tag = Tag {
            is_closing,
            buf: Bytes::new(tag_buf),
        };

        let content_len = some!(buf.iter().position(tag_open));
        let (content, mut buf) = buf.split_at(content_len);
        let content = Bytes::new(content);

        loop {
            if *some!(buf.get(1)) != b'!' {
                break;
            }
            let read = some!(buf[2..].iter().position(tag_open));
            buf = &buf[2 + read..];
        }

        self.buffer = buf;

        Ready(Ok((tag, content)))
    }
}

// ===== Tag =====

#[derive(Debug)]
pub struct Tag {
    is_closing: bool,
    buf: Bytes,
}

impl Tag {
    pub fn is_closing(&self) -> bool {
        self.is_closing
    }

    pub fn is_self_close(&self) -> bool {
        self.buf[self.buf.len() - 2] == b'/'
    }

    // pub fn name(&self) -> Bytes {
    //     Bytes::new(self.name_slice())
    // }

    pub fn name_slice(&self) -> &[u8] {
        let len = self.buf.iter().position(tag_name).expect("parser error");
        &self.buf[..len]
    }

    pub fn attrs(&self) -> Attrs {
        let len = self.buf.iter().position(tag_name).expect("parser error");
        Attrs {
            buf: Bytes::new(self.buf[len..].trim_ascii_start()),
        }
    }
}

// ===== Attributes =====

pub struct Attrs {
    buf: Bytes,
}

impl Attrs {
    pub fn next(&mut self, name: &str) -> Attr {
        if self.buf[0] == b'>' {
            panic!("end of attribute, expect: `{name}`");
        }
        match self.next_inner(&[name]) {
            Ok(ok) => ok,
            Err(err) => panic!("expect attribute `{name}` found `{err}`"),
        }
    }

    pub fn next_if(&mut self, name: &str) -> Option<Attr> {
        if self.buf[0] == b'>' {
            return None;
        }
        self.next_inner(&[name]).ok()
    }

    pub fn next_if_in(&mut self, names: &[&str]) -> Option<Attr> {
        if self.buf[0] == b'>' {
            return None;
        }
        self.next_inner(names).ok()
    }

    fn next_inner(&mut self, expect_names: &[&str]) -> Result<Attr, Bytes> {
        let mut buf = &*self.buf;
        loop {
            let [byte, rest @ ..] = buf else {
                unreachable!("{:?}", self.buf)
            };
            buf = rest;
            if *byte == b'=' {
                let len = self.buf.element_offset(byte).unwrap();
                let name = &self.buf[..len];
                if !expect_names.iter().any(|ex|ex.as_bytes() == name) {
                    return Err(Bytes::new(name));
                }
                break;
            }
        }
        loop {
            let [byte, rest @ ..] = buf else {
                unreachable!()
            };
            buf = rest;
            if *byte == b'"' {
                break;
            }
        }
        let end = loop {
            let [byte, rest @ ..] = buf else {
                unreachable!()
            };
            buf = rest;
            if *byte == b'"' {
                break byte;
            }
        };
        let len = self.buf.element_offset(end).unwrap() + 1;
        let (attr, rest) = self.buf.split_at(len);
        let attr = Bytes::new(attr);

        let mut rest = rest.trim_ascii_start();
        if rest[0] == b'/' {
            rest = &rest[1..]
        }
        self.buf = Bytes::new(rest);
        Ok(Attr { buf: attr })
    }
}

pub struct Attr {
    buf: Bytes,
}

impl Attr {
    // pub fn name(&self) -> Bytes {
    //     Bytes::new(self.name_slice())
    // }

    pub fn name_slice(&self) -> &[u8] {
        let len = self
            .buf
            .iter()
            .position(attr_name)
            .expect("no value attribute");
        &self.buf[..len]
    }

    pub fn value(&self) -> Bytes {
        Bytes::new(self.value_slice())
    }

    pub fn value_slice(&self) -> &[u8] {
        let len = self
            .buf
            .iter()
            .position(attr_name)
            .expect("no value attribute");
        &self.buf[len + 2..self.buf.len() - 1]
    }
}
