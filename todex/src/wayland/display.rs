use std::fmt;

use crate::wayland::primitives::{AsObjectId, Fixed, ObjectId, Version};
use crate::wayland::object::{NewId, Object};

// ===== AsDisplay =====

/// Formatting wayland message.
pub trait AsDisplay {
    /// Returns [`fmt::Display`] implementation which display this message.
    ///
    /// Note that currently some implementation will ignore any `fmt` flags.
    fn display(&self) -> impl fmt::Display;
}

// the rest are extensions for `Formatter`

// ===== FormatterExt =====

pub trait FormatterExt<'a, 'b> {
    fn debug_msg(&'a mut self, interface: &str, op: &str) -> DebugMsg<'a, 'b>;
}

impl<'a, 'b> FormatterExt<'a, 'b> for fmt::Formatter<'b> {
    #[inline]
    fn debug_msg(&'a mut self, interface: &str, op: &str) -> DebugMsg<'a, 'b> {
        let result = self
            .write_str(interface)
            .and_then(|_| self.write_str("::"))
            .and_then(|_| self.write_str(op))
            .and_then(|_| self.write_str("("));
        DebugMsg {
            fmt: self,
            result,
            has_fields: false,
        }
    }
}

// ===== DebugMsg =====

pub struct DebugMsg<'a, 'b> {
    fmt: &'a mut fmt::Formatter<'b>,
    result: fmt::Result,
    has_fields: bool,
}

impl<'a, 'b> DebugMsg<'a, 'b> {
    #[inline]
    pub fn field(&mut self, name: &str, value: &dyn FieldDisplay) -> &mut Self {
        self.result = self.result.and_then(|_| {
            if self.has_fields {
                self.fmt.write_str(", ")?;
            }
            self.fmt.write_str(name)?;
            self.fmt.write_str("=")?;
            value.fmt(self.fmt)
        });
        self.has_fields = true;
        self
    }

    #[inline]
    pub fn finish(&mut self) -> fmt::Result {
        self.result = self.result.and_then(|_| self.fmt.write_str(")"));
        self.result
    }
}

// ===== FieldDisplay =====

/// Formattable message field.
pub trait FieldDisplay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result;
}

macro_rules! delegate_display {
    ($ty:ty) => {
        impl FieldDisplay for $ty {
            #[inline]
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                fmt::Display::fmt(self, f)
            }
        }
    };
}

delegate_display!(u32);
delegate_display!(i32);
delegate_display!(ObjectId);
delegate_display!(Fixed);
delegate_display!(Version);

impl<I> FieldDisplay for NewId<I> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self, f)
    }
}

impl<I, M> FieldDisplay for Object<I, M> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.object_id(), f)
    }
}

impl<I, M> FieldDisplay for Option<Object<I, M>> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Some(o) => o.fmt(f),
            None => f.write_str("<none>"),
        }
    }
}

impl FieldDisplay for &str {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use fmt::Write as _;
        f.write_char('"')?;
        f.write_str(self)?;
        f.write_char('"')
    }
}

impl FieldDisplay for Option<&str> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Some(s) => s.fmt(f),
            None => f.write_str("<none>"),
        }
    }
}

impl FieldDisplay for &[u8] {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("<bytes>")
    }
}
