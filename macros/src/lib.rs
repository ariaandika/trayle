use proc_macro::*;

use crate::codegen::generate;
use crate::error::Error;
use crate::parser::Parser;
use crate::syntax::Attributes;

mod parser;
mod syntax;
mod codegen;
mod error;

/// Implement `FromObjectId`, `AsObjectId` and `AsInterface`.
#[proc_macro_derive(Interface)]
pub fn interface(tokens: TokenStream) -> TokenStream {
    impl_interface(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

fn impl_interface(mut parser: Parser) -> Result<TokenStream, Error> {
    let _attrs = parser.parse::<Attributes>()?;
    let _vis = parser.ident_of("pub")?;
    let _struct_kw = parser.ident_of("struct")?;
    let name = parser.ident()?;

    Ok(codegen::generate! {
        impl FromObjectId for #&name {
            #[inline]
            fn from_object_id(id: ObjectId) -> Self {
                Self { id }
            }
        }
        impl AsObjectId for #&name {
            #[inline]
            fn object_id(&self) -> ObjectId {
                self.id
            }
        }
        impl AsInterface for #&name {
            const INTERFACE: Interface = Interface::#name;
        }
    })
}

/// Implement `OpCode`.
#[proc_macro_derive(OpCode)]
pub fn op_code(tokens: TokenStream) -> TokenStream {
    impl_op_code(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

fn impl_op_code(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let mut body = Parser::new(parser.group_of(Delimiter::Brace)?.stream());

    let zero = Literal::u8_unsuffixed(0);

    let mut i = 1;
    let mut last_variant = body.next_ident().expect("empty enum");
    loop {
        body.next_punct_of(',');
        let Some(next_ident) = body.next_ident() else {
            break;
        };
        last_variant = next_ident;
        i += 1;
    }
    let from_op = match i {
        1 => generate! {
            if op == #zero {
                Ok(Self::#last_variant)
            } else {
                Err(WlError::UnknownOp)
            }
        },
        _ => generate! {
            if op as u8 <= Self::#last_variant as u8 {
                Ok(unsafe { std::mem::transmute::<u8, Self>(op as u8) })
            } else {
                Err(WlError::UnknownOp)
            }
        },
    };

    Ok(generate! {
        impl OpCode for #name {
            #[inline]
            fn from_op(op: u16) -> Result<Self, WlError> {
                #from_op
            }

            #[inline]
            fn to_op(self) -> u16 {
                self as u16
            }
        }
    })
}
