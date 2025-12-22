use std::mem::transmute;
use std::ptr::copy_nonoverlapping;

macro_rules! position {
    ($buf:expr, whs | $b:pat) => {
        $buf.iter()
            .position(|e| e.is_ascii_whitespace() || matches!(e, $b))
            .unwrap()
    };
    ($buf:expr, $b:pat) => {
        $buf.iter()
            .position(|e| matches!(e, $b))
            .unwrap()
    };
    ($buf:expr, $b:pat, $expect:expr) => {
        $buf.iter()
            .position(|e| matches!(e, $b))
            .expect($expect)
    };
}

macro_rules! position_opt {
    ($buf:expr, whs | $b:pat) => {
        $buf.iter()
            .position(|e| e.is_ascii_whitespace() || matches!(e, $b))
    };
    ($buf:expr, $b:pat) => {
        $buf.iter()
            .position(|e| matches!(e, $b))
    };
}

pub struct Tag {
    read: usize,
    buffer: Vec<u8>,
}

/// Pull based parser.
pub struct Parser {
    read: usize,
    buffer: Vec<u8>,
    io: Box<dyn std::io::Read>,
}

impl Tag {
    /// Returns `(name, is_closing)`.
    pub fn name(&mut self) -> (Vec<u8>, bool) {
        assert_eq!(self.buf()[0], b'<');
        let is_closing = self.buf()[1] == b'/';
        self.read += 1 + is_closing as usize;

        let len = position!(self.buf(), whs | b'>' | b'/');
        let name = self.split_at(len);
        self.skip_wh();

        (name, is_closing)
    }

    pub fn next_attr(&mut self) -> Option<(Vec<u8>, Vec<u8>)> {
        if matches!(self.buf()[0], b'/' | b'>') {
            return None;
        }

        assert!(self.buf()[0].is_ascii_alphabetic());
        let len = position!(self.buf(), b'=');
        let name = self.split_at(len);

        assert_eq!(self.buf()[1], b'"');
        self.read += 2; // ="

        let len = position!(self.buf(), b'"');
        let mut value = self.split_at(len + 1);
        value.pop();
        self.skip_wh();

        Some((name, value))
    }

    pub fn is_self_close(&mut self) -> bool {
        loop {
            match self.buf()[0] {
                b'/' => return true,
                b'>' => return false,
                _ => {
                    self.next_attr();
                },
            }
        }
    }

    fn buf(&self) -> &[u8] {
        &self.buffer[self.read..]
    }

    fn split_at(&mut self, at: usize) -> Vec<u8> {
        let read = self.read;
        self.read += at;
        self.buffer[read..read + at].to_vec()
    }

    fn skip_wh(&mut self) {
        while let Some(&b) = self.buf().first() {
            if b.is_ascii_whitespace() {
                self.read += 1;
            } else {
                break
            }
        }
    }
}

impl Parser {
    pub fn new<IO: std::io::Read + 'static>(io: IO) -> Self {
        Self { read: 0, buffer: Vec::new(), io: Box::new(io) }
    }

    pub fn assert_prolog(&mut self) {
        let tag = self.next_tag_buf();
        assert_eq!(tag, b"<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
    }

    pub fn is_tag(&self) -> bool {
        self.buf()[0] == b'<'
    }

    pub fn next_tag(&mut self) -> Tag {
        Tag { read: 0, buffer: self.next_tag_buf() }
    }

    pub fn peek_tag(&mut self) -> (Vec<u8>, bool) {
        assert!(self.is_tag(), "{}",str::from_utf8(self.buf()).unwrap());

        let is_closing = self.buf()[1] == b'/';
        let len = if is_closing {
            // position!(self.buf(), whs | b'>')
            let Some(len) = position_opt!(self.buf(), whs | b'>') else {
                self.read();
                return self.peek_tag()
            };
            len
        } else {
            // position!(self.buf(), whs | b'>' | b'/')
            let Some(len) = position_opt!(self.buf(), whs | b'>' | b'/') else {
                self.read();
                return self.peek_tag()
            };
            len
        };

        let name = &self.buf()[1 + is_closing as usize..len];

        if name != b"!--" {
            (name.to_vec(), is_closing)
        } else {
            let Some(len) = position_opt!(self.buf(), b'>') else {
                self.read();
                return self.peek_tag()
            };
            let _ = self.split_at(len + 1);
            self.skip_wh();
            self.peek_tag()
        }
    }

    pub fn next_plain(&mut self) -> Vec<u8> {
        let Some(len) = position_opt!(self.buf(), b'<') else {
            self.read();
            return self.next_plain()
        };
        let mut plain = self.split_at(len);
        plain.truncate(plain.trim_ascii_end().len());
        plain
    }

    // ===== private =====

    fn buf(&self) -> &[u8] {
        &self.buffer[self.read..]
    }

    fn split_at(&mut self, at: usize) -> Vec<u8> {
        let read = self.read;
        self.read += at;
        self.buffer[read..read + at].to_vec()
    }

    fn next_tag_buf(&mut self) -> Vec<u8> {
        if self.buf().is_empty() {
            self.read();
        }

        assert_eq!(
            self.buf()[0],
            b'<',
            "expected tag, but not in open bracket {:?}",
            str::from_utf8(self.buf()).unwrap()
        );

        let Some(len) = position_opt!(self.buf(), b'>') else {
            self.read();
            return self.next_tag_buf();
        };
        let lead = self.split_at(len + 1);
        self.skip_wh();

        if lead[1] != b'!' {
            lead.to_vec()
        } else {
            assert_eq!(&lead[2..4], b"--");
            assert_eq!(&lead[lead.len() - 3..], b"-->");
            self.next_tag_buf()
        }
    }

    fn skip_wh(&mut self) {
        while let Some(&b) = self.buf().first() {
            if b.is_ascii_whitespace() {
                self.read += 1;
            } else {
                break
            }
        }
    }

    fn read(&mut self) {
        self.reserve();

        unsafe {
            let spare: &mut [u8] = transmute(self.buffer.spare_capacity_mut());

            assert_ne!(spare.len(), 0);

            let read = self.io.read(spare).unwrap();
            if read == 0 {
                panic!("end of file");
            }

            self.buffer.set_len(self.buffer.len() + read);
        };

        self.skip_wh();
    }

    fn reserve(&mut self) {
        let len = self.buf().len();

        if self.read > self.buffer.capacity() / 2 {
            unsafe {
                copy_nonoverlapping(
                    self.buffer.as_ptr().add(self.read),
                    self.buffer.as_mut_ptr(),
                    len
                );
                self.buffer.set_len(len);
            }
        } else {
            assert!(len <= 8 * 1024);
            let mut vec = Vec::with_capacity(8 * 1024);
            unsafe {
                let spare: &mut [u8] = transmute(vec.spare_capacity_mut());
                copy_nonoverlapping(
                    self.buffer.as_ptr().add(self.read),
                    spare.as_mut_ptr(),
                    len
                );
                vec.set_len(len);
            }
            self.buffer = vec;
        }


        self.read = 0;
    }
}
