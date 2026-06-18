use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let attrs = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("struct")?;
    let name = parser.ident()?;

    let wl_name = Literal::string(&to_snake(name.as_str()));
    let global = match attrs.find_seq::<Global>("global")? {
        Some((_, Global { version })) => token_stream! {
            impl AsGlobal for #name {
                const NAME: &str = #wl_name;

                const VERSION: u32 = #version;

                const INTERFACE: Interface = Interface::#name;
            }
        },
        None => token_stream!(),
    };

    Ok(generate! {
        impl FromObjectId for #name {
            #[inline]
            fn from_object_id(id: ObjectId) -> Self {
                Self { id }
            }
        }

        impl AsObjectId for #name {
            #[inline]
            fn object_id(&self) -> ObjectId {
                self.id
            }
        }

        impl AsInterface for #name {
            #[inline]
            fn interface(&self) -> Interface {
                Interface::#name
            }
        }

        @global
    }.collect())
}

struct Global {
    version: Literal,
}

impl Parse for Global {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        Ok(Self { version: parser.lit()? })
    }
}
