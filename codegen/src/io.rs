
pub trait Write {
    fn write_fmt(&mut self, args: std::fmt::Arguments);
}

impl<W: std::io::Write> Write for W {
    fn write_fmt(&mut self, args: std::fmt::Arguments) {
        std::io::Write::write_fmt(self, args).expect("write error");
    }
}

