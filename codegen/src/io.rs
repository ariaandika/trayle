
pub trait Write {
    fn write_fmt(&mut self, args: std::fmt::Arguments);
}

impl<W: std::fmt::Write> Write for W {
    fn write_fmt(&mut self, args: std::fmt::Arguments) {
        std::fmt::Write::write_fmt(self, args).expect("failed to write");
    }
}
