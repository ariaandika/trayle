use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("struct")?;
    let name = parser.ident()?;

    Ok(generate! {
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
