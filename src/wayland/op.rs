use crate::wayland::WlError;

// ===== trait =====

pub trait OpCode: Sized {
    fn from_op(op: u16) -> Result<Self, WlError>;

    fn to_op(self) -> u16;
}

macro_rules! opcode {
    (pub enum $name:ident { $($v:ident,)* }) => {
        #[derive(Debug, Clone, Copy)]
        pub enum $name {
            $($v,)*
        }

        impl OpCode for $name {
            #[allow(nonstandard_style)]
            #[inline]
            fn from_op(op: u16) -> Result<Self, WlError> {
                $(const $v: u16 = $name::$v as u16;)*

                match op {
                    $($v => Ok(Self::$v),)*
                    _ => Err(WlError::UnknownOp),
                }
            }

            #[inline]
            fn to_op(self) -> u16 {
                self as u16
            }
        }
    };
}

pub(crate) use opcode;

