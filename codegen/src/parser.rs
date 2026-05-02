use std::cmp;
use std::task::Poll::{self, *};

use crate::buffer::{FileBuffer, Str};

// ===== Parser =====

pub struct Parser {
    buffer: FileBuffer,
}

impl Parser {
    pub fn new(buffer: FileBuffer) -> Self {
        assert_eq!(buffer.first(), Some(b'<'));
        Self { buffer }
    }
}

impl Parser {
    pub fn assert_prolog(&mut self, prolog: &str) {
        loop {
            let Some(slice) = self.buffer.get(..prolog.len()) else {
                self.buffer.read();
                continue;
            };
            assert_eq!(slice, prolog.as_bytes());
            self.buffer.advance(prolog.len());
            self.buffer.trim_ascii_start_mut();
            break;
        }
    }

    pub fn next_tag(&mut self, name: &str) -> (Tag, Str) {
        let name_bytes = self.peek();
        if name_bytes != name.as_bytes() {
            panic!("expected `{name}` tag, found `{}`", String::from_utf8_lossy(name_bytes))
        }
        self.next_tag_inner()
    }

    pub fn next_tag_if(&mut self, name: &str) -> Option<(Tag, Str)> {
        if self.peek() != name.as_bytes() {
            return None;
        }
        Some(self.next_tag_inner())
    }
}

/// find whitespace or `'>'`.
fn tag_name(b: &u8) -> bool {
    // matches!(*self, b'\t' | b'\n' | b'\x0C' | b'\r' | b' ')
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

/// whitespace or `'>'`.
static TAG_NAME_DELIM: [char; 6] = ['\t', '\n', '\x0C', '\r', ' ', '>'];

/// `Some` or returns `Pending`.
macro_rules! some {
    ($e:expr) => {
        match $e {
            Some(ok) => ok,
            None => return Pending,
        }
    };
}

impl Parser {
    pub fn peek(&mut self) -> &[u8] {
        loop {
            let Some(ok) = self.buffer.split_first() else {
                self.buffer.read();
                continue;
            };
            let (b'<', bytes) = ok else {
                unreachable!("internal error: not in `<` boundary")
            };
            let Some(name_len) = bytes.iter().position(tag_name) else {
                self.buffer.read();
                continue;
            };
            let is_closing = bytes[0] == b'/';
            let name = &bytes[is_closing as usize..name_len];
            // SAFETY: fuck off marbles
            return unsafe { std::mem::transmute::<&[u8], &[u8]>(name) };
        }
    }

    fn next_tag_inner(&mut self) -> (Tag, Str) {
        loop {
            match self.poll_next_tag_inner() {
                Ready(ok) => return ok,
                Pending => self.buffer.read(),
            }
        }
    }

    fn poll_next_tag_inner(&mut self) -> Poll<(Tag, Str)> {
        let (b'<', bytes) = some!(self.buffer.split_first()) else {
            unreachable!("internal error: not in `<` boundary")
        };
        let is_closing = bytes[0] == b'/';

        // get the tag
        let tag_len = some!(bytes.iter().position(tag_close)) + 1;

        // get plain content
        let bytes = &bytes[tag_len..];
        let content_len = some!(bytes.iter().position(tag_open));

        // skip comment, if there is comment mixed with plain content, well shiver my timber
        let mut skip_len = 0;
        loop {
            let bytes = &bytes[content_len + skip_len..];
            if *some!(bytes.get(1)) != b'!' {
                break;
            }
            let read = some!(bytes[2..].iter().position(tag_open));
            skip_len += 2 + read;
        }

        // no more pending
        self.buffer.advance(1); // skip '<'
        let tag = self.buffer.split_to(tag_len);
        let content = self.buffer.split_to(content_len);
        self.buffer.advance(skip_len);

        let tag = Tag {
            is_closing,
            string: tag,
        };
        Ready((tag, content))
    }
}

// ===== Tag =====

#[derive(Debug)]
pub struct Tag {
    is_closing: bool,
    string: Str,
}

impl Tag {
    pub fn is_closing(&self) -> bool {
        self.is_closing
    }

    pub fn is_self_close(&self) -> bool {
        self.string.as_bytes()[self.string.len() - 2] == b'/'
    }

    // pub fn name(&self) -> Bytes {
    //     Bytes::new(self.name_slice())
    // }

    // pub fn name_str(&self) -> &str {
    //     let len = self.string.find(TAG_NAME_DELIM).expect("parser error");
    //     &self.string[..len]
    // }

    pub fn attrs(&self) -> Attrs {
        let len = self.string.find(TAG_NAME_DELIM).expect("parser error");
        let mut string = self.string.slice(len..);
        string.trim_ascii_start();
        Attrs { string, }
    }
}

// ===== Attributes =====

pub struct Attrs {
    string: Str,
}

impl Attrs {
    pub fn next(&mut self, name: &str) -> Attr {
        if *self.string.first().unwrap() == b'>' {
            panic!("end of attribute, expect: `{name}`");
        }
        if let peek = self.peek().unwrap() && peek != name {
            panic!("expect attribute `{name}` found `{peek}`");
        }
        self.next_inner()
    }

    pub fn next_if(&mut self, name: &str) -> Option<Attr> {
        if *self.string.first().unwrap() == b'>' {
            return None;
        }
        if self.peek()? != name {
            return None;
        }
        Some(self.next_inner())
    }

    pub fn peek(&self) -> Option<&str> {
        if *self.string.first().unwrap() == b'>' {
            return None;
        }
        Some(self.string.split_once('=').expect("no value attr").0)
    }

    fn next_inner(&mut self) -> Attr {
        let len = self.string.find('"').expect("no value attr");
        let len = self.string[len + 1..].find('"').expect("unclosed value quote") + len + 2;
        let string = self.string.split_to(len);
        self.string.trim_ascii_start();
        if self.string.starts_with('/') {
            self.string.advance(1);
        }
        Attr { string }
    }
}

pub struct Attr {
    string: Str,
}

impl Attr {
    // pub fn name(&self) -> Bytes {
    //     Bytes::new(self.name_slice())
    // }
    //
    // pub fn name_str(&self) -> &str {
    //     let len = self.string.find('=').expect("no value attribute");
    //     &self.string[..len]
    // }

    pub fn value(&self) -> Str {
        let len = self
            .string
            .find('=')
            .expect("no value attribute");
        self.string.slice(len + 2..self.string.len() - 1)
    }

    pub fn value_str(&self) -> &str {
        let len = self
            .string
            .find('=')
            .expect("no value attribute");
        &self.string[len + 2..self.string.len() - 1]
    }
}

impl std::fmt::Debug for Parser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Parser")
            .field(
                "buffer",
                &&self.buffer[..cmp::min(self.buffer.len(), 512)],
            )
            .finish()
    }
}
