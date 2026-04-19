use std::mem;
use std::ptr::copy_nonoverlapping;

// ===== Parser =====

/// Pull based parser.
pub struct Parser {
    read: usize,
    buffer: Vec<u8>,
    io: Box<dyn std::io::Read>,
}

impl Parser {
    pub fn new<IO: std::io::Read + 'static>(io: IO) -> Self {
        let mut me = Self {
            read: 0,
            buffer: Vec::with_capacity(1024),
            io: Box::new(io),
        };
        let prolog = me.next_tag();
        assert_eq!(prolog.buf, b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
        me
    }

    #[cfg(test)]
    fn new_test<IO: std::io::Read + 'static>(io: IO) -> Self {
        Self {
            read: 0,
            buffer: Vec::with_capacity(1024),
            io: Box::new(io),
        }
    }
}

impl Parser {
    fn buf(&self) -> &[u8] {
        &self.buffer[self.read..]
    }

    pub fn next_tag(&mut self) -> Tag<'_> {
        let len = self.peek_tag_inner();
        let buf = &self.buffer[self.read..self.read + len];
        self.read += len;
        Tag { buf }
    }

    pub fn peek_tag(&mut self) -> Tag<'_> {
        let len = self.peek_tag_inner();
        let buf = &self.buffer[self.read..self.read + len];
        Tag { buf }
    }

    fn peek_tag_inner(&mut self) -> usize {
        // find `<`
        if self.buf().first() != Some(&b'<') {
            loop {
                let Some(start) = self.buf().iter().position(|e|*e == b'<') else {
                    self.read();
                    continue;
                };
                let Some(delim) = self.buf().get(start + 1) else {
                    self.read();
                    continue;
                };

                if *delim == b'!' {
                    // skip comment
                    match self.buf()[start..].iter().position(|e|*e == b'>') {
                        Some(len) => self.read += start + len,
                        None => self.read(),
                    }
                    continue;
                }

                self.read += start;
                break;
            }
        }

        // find `>`
        loop {
            match self.buf().iter().position(|e|*e == b'>') {
                Some(close) => break close + 1,
                None => {
                    self.read();
                    continue;
                },
            }
        }
    }

    pub fn next_plain(&mut self) -> &[u8] {
        // find `<`
        loop {
            let Some(end) = self.buf().iter().position(|e|*e == b'<') else {
                self.read();
                continue;
            };
            let Some(delim) = self.buf().get(end + 1) else {
                self.read();
                continue;
            };

            if *delim == b'!' {
                // skip comment
                match self.buf()[end..].iter().position(|e|*e == b'>') {
                    Some(len) => self.read += end + len,
                    None => self.read(),
                }
                continue;
            }

            let buf = &self.buffer[self.read..self.read + end];
            self.read += end;
            return buf;
        }
    }
}

// ===== Allocation =====

impl Parser {
    fn read(&mut self) {
        self.reserve();

        unsafe {
            let spare: &mut [u8] = mem::transmute(self.buffer.spare_capacity_mut());

            assert_ne!(spare.len(), 0);

            let read = self.io.read(spare).unwrap();
            if read == 0 {
                panic!("end of file");
            }

            self.buffer.set_len(self.buffer.len() + read);
        }
    }

    fn reserve(&mut self) {
        let len = self.buf().len();

        if self.read > self.buffer.capacity() / 2 {
            // if the remaining data can be copied backward without overlapping, skip allocating
            unsafe {
                let ptr = self.buffer.as_mut_ptr();
                copy_nonoverlapping(ptr.add(self.read), ptr, len);
            }
        } else {
            let mut vec = Vec::with_capacity(self.buffer.capacity() << 1);
            unsafe {
                let spare: &mut [u8] = mem::transmute(vec.spare_capacity_mut());
                copy_nonoverlapping(self.buffer.as_ptr().add(self.read), spare.as_mut_ptr(), len);
            }
            self.buffer = vec;
        }

        unsafe { self.buffer.set_len(len) };
        self.read = 0;
    }
}

// ===== Tag =====

pub struct Tag<'a> {
    buf: &'a [u8],
}

impl<'a> Tag<'a> {
    pub fn is_closing(&self) -> bool {
        self.buf[1] == b'/'
    }

    pub fn is_self_close(&self) -> bool {
        self.buf[self.buf.len() - 2] == b'/'
    }

    pub fn name(&self) -> &'a [u8] {
        let len = self
            .buf
            .iter()
            .position(|e| *e == b'>' || e.is_ascii_whitespace())
            .expect("unclosed tag");
        let end_delim = (self.buf[1] == b'/') as usize;
        &self.buf[1 + end_delim..len]
    }

    pub fn attrs(&self) -> Attrs<'a> {
        let len = self
            .buf
            .iter()
            .position(|e| *e == b'>' || e.is_ascii_whitespace())
            .expect("unclosed tag");
        assert_ne!(self.buf[len], b'>', "no attribute");
        let end_delim = (self.buf[1] == b'/') as usize;
        Attrs {
            buf: &self.buf[1 + end_delim + len..self.buf.len() - 1],
        }
    }
}

// ===== Attributes =====

pub struct Attrs<'a> {
    buf: &'a [u8],
}

impl<'a> Attrs<'a> {
    pub fn next(&mut self) -> Attr<'a> {
        self.try_next().expect("no attribute remaining")
    }

    pub fn try_next(&mut self) -> Option<Attr<'a>> {
        let len = self.peek_inner()?;
        let (buf, rest) = std::mem::take(&mut self.buf).split_at(len);
        let whs = rest
            .iter()
            .position(|e| e.is_ascii_whitespace())
            .map(|e|e + 1)
            .unwrap_or(rest.len());
        self.buf = &rest[whs..];
        Some(Attr { buf })
    }

    fn peek_inner(&mut self) -> Option<usize> {
        if self.buf.is_empty() {
            return None;
        }
        let name_len = self
            .buf
            .iter()
            .position(|e| *e == b'=')?;
        assert_eq!(self.buf[name_len + 1], b'"', "unquoted attribute");
        let len = self.buf[name_len + 2..]
            .iter()
            .position(|e| *e == b'"')
            .expect("unclosed attr quote");
        Some(name_len + 2 + len + 1)
    }
}

pub struct Attr<'a> {
    buf: &'a [u8],
}

impl<'a> Attr<'a> {
    pub fn name(&self) -> &'a [u8] {
        let len = self
            .buf
            .iter()
            .position(|e| *e == b'=')
            .expect("no value attribute");
        &self.buf[..len]
    }

    pub fn value(&self) -> &'a [u8] {
        let off = self
            .buf
            .iter()
            .position(|e| *e == b'=')
            .expect("no value attribute");
        &self.buf[off + 2..self.buf.len() - 1]
    }
}

#[test]
fn test_parser() {
    const BUF: &[u8] = b"Nice <!-- lmao --><tag control>";

    let mut parser = Parser::new_test(BUF);
    assert_eq!(parser.peek_tag().buf, b"<tag control>");

    let tag = parser.next_tag();
    assert_eq!(tag.buf, b"<tag control>");
    assert_eq!(tag.name(), b"tag");
    assert_eq!(parser.buf(), b"");

    // ===== Attr =====

    const BUF2: &[u8] = b"<description summary=\"foo bar baz\" then=\"baz bar foo\">";

    let mut parser = Parser::new_test(BUF2);
    let tag = parser.next_tag();

    let mut attrs = tag.attrs();

    let attr = attrs.next();
    assert_eq!(attr.name(), b"summary");
    assert_eq!(attr.value(), b"foo bar baz");

    let attr = attrs.next();
    assert_eq!(attr.name(), b"then");
    assert_eq!(attr.value(), b"baz bar foo");
}
