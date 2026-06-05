use proc_macro::*;

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
