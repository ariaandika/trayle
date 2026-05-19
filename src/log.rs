pub trait Log {
    fn subject(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;

    fn info(&self, msg: &str) {
        println!("[{:?}] {msg}", std::fmt::from_fn(|f| self.subject(f)));
    }

    fn write_fmt(&self, args: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        use std::io::Write;
        let mut stdout = std::io::stdout().lock();
        stdout.write_fmt(format_args!(
            "[{:?}] {args}",
            std::fmt::from_fn(|f| self.subject(f))
        ))
    }
}

macro_rules! simple_log {
    ($t:ty, $n:literal) => {
        impl Log for $t {
            fn subject(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt($n, f)
            }
        }
    };
}

simple_log!(crate::epoll::Epoll, "EPOLL");
simple_log!(crate::sigfd::Sigfd, "SIGNALFD");

impl Log for crate::clients::Client {
    fn subject(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (idx, id) = self.id().to_parts();
        write!(f, "CLIENT{{idx={idx},id={id}}}")
    }
}

impl Log for crate::clients::ClientMut<'_> {
    fn subject(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let (idx, id) = self.id().to_parts();
        write!(f, "CLIENT{{idx={idx},id={id}}}")
    }
}
