use crate::prelude::*;

pub struct Interface {
    pub iface: Ident,
    pub wl_iface: Ident,
    pub global: Option<Literal>,
}

impl Parse for Interface {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut global = None;

        if parser.next_punct_of('#').is_some() {
            let mut attr = parser.group_of(Delimiter::Bracket)?.body_parser();

            if attr.next_ident_of("global").is_some() {
                attr.punct_of('=')?;
                global = Some(attr.parse()?);
                attr.next_punct_of(',');
            }

            attr.check_empty()?;
        }

        parser.next_ident_of("pub");
        parser.ident_of("struct")?;
        let iface = parser.parse::<Ident>()?;
        parser.punct_of(';')?;

        let wl_iface = iface.to_snake().spanned(Span::call_site());

        Ok(Self {
            iface,
            wl_iface,
            global,
        })
    }
}

impl Interface {
    pub fn gen_struct(&self) -> impl Iterator<Item = TokenTree> {
        let iface_name = &self.iface;
        g! {
            #[derive(Debug, Default, Clone, Copy)]
            pub struct #iface_name(std::marker::PhantomData<()>);
        }
    }

    pub fn gen_impl_marker(&self) -> impl Iterator<Item = TokenTree> {
        let iface_name = &self.iface;
        g! {
            impl InterfaceMarker for #iface_name {
                fn from_interface(iface: InterfaceId) -> Self {
                    assert_iface!(iface, #iface_name);
                    <Self as sealed::Sealed>::MARKER
                }
            }
            impl sealed::Sealed for #iface_name {
                const MARKER: Self = Self(std::marker::PhantomData);
            }
        }
    }

    pub fn gen_wl_interface(&self) -> impl Iterator<Item = TokenTree> {
        let iface_name = &self.iface;
        g! {
            impl WlInterface for #iface_name {
                type RequestOp = RequestOp;

                type EventOp = EventOp;
            }
        }
    }

    pub fn gen_wl_global(&self) -> impl Iterator<Item = TokenTree> {
        self.global.as_ref().map_stream(|version|{
            let iface_name = &self.iface;
            let wl_string = Literal::string(self.wl_iface.as_str());
            g! {
                impl WlGlobal for #iface_name {
                    const NAME: &str = #wl_string;
                    const VERSION: Version = Version::new(#version).unwrap();
                    const INTERFACE: Interface = Interface::#iface_name;
                }
            }
        })
    }
}

