use crate::prelude::*;

pub struct Interface {
    pub iface_name: Ident,
    pub iface_span: Span,
    pub wl_string: Literal,
    pub global: Option<Literal>,
    pub mod_name: Option<Ident>,
}

impl Parse for Interface {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut global = None;
        let mut mod_name = Some(());

        if parser.next_punct_of('#').is_some() {
            let mut attr = parser.group_of(Delimiter::Bracket)?.body_parser();

            if attr.next_ident_of("global").is_some() {
                attr.punct_of('=')?;
                global = Some(attr.parse()?);
                attr.next_punct_of(',');
            }

            if attr.next_ident_of("no_mod").is_some() {
                mod_name = None;
                attr.next_punct_of(',');
            }

            attr.check_empty()?;
        }

        parser.next_ident_of("pub");
        parser.ident_of("struct")?;
        let mut iface_name = parser.parse::<Ident>()?;
        parser.punct_of(';')?;

        let iface_span = iface_name.unspan();
        let wl_string = iface_name.to_lit_snake();
        let mod_name = mod_name.map(|_|iface_name.to_snake());

        Ok(Self {
            iface_name,
            iface_span,
            wl_string,
            global,
            mod_name,
        })
    }
}

impl Interface {
    pub fn generate(&self) -> impl Iterator<Item = TokenTree> {
        let iface_name_spanned = self.iface_name.clone().spanned(self.iface_span);
        let Self { iface_name, wl_string, .. } = self;

        let global = self.global.as_ref().map_stream(|version|{
            g! {
                impl WlGlobal for #iface_name {
                    const NAME: &str = #wl_string;
                    const VERSION: Version = Version::new(#version).unwrap();
                    const INTERFACE: Interface = Interface::#iface_name;
                }
            }
        });

        g! {
            #[derive(Debug, Default, Clone, Copy)]
            pub struct #iface_name_spanned(());

            pub type InterfaceType = #iface_name;

            impl sealed::Sealed for #iface_name {
                const MARKER: Self = Self(());
            }

            impl WlInterface for #iface_name {
                type RequestOp = RequestOp;

                type EventOp = EventOp;

                const INTERFACE_NAME: &str = #wl_string;

                fn try_from_interface(iface: InterfaceId) -> Option<Self> {
                    if iface == Interface::#iface_name {
                        Some(<Self as sealed::Sealed>::MARKER)
                    } else {
                        None
                    }
                }
            }

            @global
        }
    }
}
