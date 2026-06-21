use crate::wayland::{AsObjectId, Message, Fixed, NewId, Object, ObjectId, Version};

#[inline]
pub fn fmt_me<D: Display2>(value: &D, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    D::fmt(value, f)
}

pub trait AsDisplay {
    fn display(&self) -> impl std::fmt::Display;
}

impl<T: AsDisplay> AsDisplay for Message<T> {
    fn display(&self) -> impl std::fmt::Display {
        self.payload.display()
    }
}

// separate trait to implement display for `Option` type.

pub trait Display2 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result;
}

macro_rules! delegate {
    ($me:ident$(<$t:ident>)?) => {
        impl$(<$t>)? Display2 for $me$(<$t>)? {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }
    };
}

delegate!(u32);
delegate!(i32);
delegate!(ObjectId);
delegate!(NewId<T>);
delegate!(Fixed);
delegate!(Version);

impl Display2 for Option<ObjectId> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Some(ok) => ok.fmt(f),
            None => "<none>".fmt(f),
        }
    }
}

impl<T: AsObjectId> Display2 for Object<T> {
    #[inline]
    fn fmt(&self,f: &mut std::fmt::Formatter) -> std::fmt::Result {
        self.object_id().fmt(f)
    }
}

impl<T: AsObjectId> Display2 for Option<Object<T>> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Some(ok) => ok.object_id().fmt(f),
            None => "<none>".fmt(f),
        }
    }
}

impl Display2 for &str {
    #[inline]
    fn fmt(&self,f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "\"{self}\"")
    }
}

impl Display2 for &[u8] {
    #[inline]
    fn fmt(&self,f: &mut std::fmt::Formatter) -> std::fmt::Result {
        "<array>".fmt(f)
    }
}

impl Display2 for Option<&str> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Some(ok) => write!(f, "\"{ok}\""),
            None => "<none>".fmt(f),
        }
    }
}
