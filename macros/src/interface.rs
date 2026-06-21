use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let attr = parser.parse::<InterfaceAttr>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("struct")?;
    let name = parser.ident()?;

    let base = generate! {
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
    };

    Ok(attr.generate(&name).chain(base).collect())
}

struct InterfaceAttr {
    global: Option<Literal>,
    data: Option<Ident>,
}

impl Parse for InterfaceAttr {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut seq = SequenceAttr::parse_attrs_opt("interface", parser)?;
        Ok(Self {
            global: seq.next_named_of("global")?,
            data: seq.next_named_of("data")?,
        })
    }
}

impl InterfaceAttr {
    fn generate(self, name: &Ident) -> impl Iterator<Item = TokenTree> {
        let Self { global, data } = self;
        let wl_name = Literal::string(&to_snake(name.as_str()));

        let global = global.map_stream(move |version| generate! {
            impl WlGlobal for #name {
                const NAME: &str = #wl_name;
                const VERSION: Version = Version::new(#version).unwrap();
                const INTERFACE: Interface = Interface::#name;
            }
        });
        let data = data.map_stream(|data| generate! {
            impl AsObjectData for #name {
                type Data = #data;
            }
        });

        global.chain(data)
    }
}
