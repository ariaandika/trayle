use crate::prelude::*;

pub struct Interface {
    pub iface_name: Ident,
    pub iface_span: Span,
    pub wl_iface: Ident,
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
        let wl_iface = iface_name.to_snake();
        let mod_name = mod_name.map(|_|wl_iface.clone());

        Ok(Self {
            iface_name,
            iface_span,
            wl_iface,
            global,
            mod_name,
        })
    }
}

impl Interface {
    pub fn generate(&self) -> impl Iterator<Item = TokenTree> {
        let iface_name_spanned = self.iface_name.clone().spanned(self.iface_span);
        let iface_name = &self.iface_name;

        let global = self.global.as_ref().map_stream(|version|{
            let wl_string = Literal::string(self.wl_iface.as_str());
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

            impl InterfaceMarker for #iface_name {
                fn from_interface(iface: InterfaceId) -> Self {
                    assert_iface!(iface, #iface_name);
                    <Self as sealed::Sealed>::MARKER
                }
            }

            impl sealed::Sealed for #iface_name {
                const MARKER: Self = Self(());
            }

            impl WlInterface for #iface_name {
                type RequestOp = RequestOp;

                type EventOp = EventOp;
            }

            @global
        }
    }
}
