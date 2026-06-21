use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let EnumItem { name, body, .. } = parser.parse()?;

    let mut i = 0u32;
    let mut has_discr = None;

    let mut body = body.body_parser();
    let mut name_getter = GenNameGetter::new(&name);
    let mut wl_enum = GenWlEnum::new(&name);

    while let Some(Variant { ident, discr, .. }) = body.punctuated(',')? {
        let discr = match discr {
            Some(ok) => {
                if has_discr.is_none() {
                    has_discr = Some(ident.span());
                }
                ok.expr
            }
            None => {
                let lit = Literal::u32_unsuffixed(i);
                i += 1;
                generate!(#lit).collect()
            }
        };
        name_getter.add_field(&ident);
        wl_enum.add_field(&ident, discr);
    }

    if let Some(custom_lit) = has_discr
        && i != 0
    {
        return Err(Error::spanned(
            "custom value must be all variant or nothing",
            custom_lit,
        ));
    }

    let display = generate! {
        impl std::fmt::Display for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }
        impl display::Display2 for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }
    };

    Ok(display
        .chain(name_getter.gens())
        .chain(wl_enum.gens())
        .collect())
}

// ===== Components =====

struct GenNameGetter {
    name: Ident,
    arms: TokenStream,
}

impl GenNameGetter {
    fn new(name: &Ident) -> Self {
        Self {
            name: name.clone(),
            arms: TokenStream::new(),
        }
    }

    fn add_field(&mut self, variant: &Ident) {
        let wl_variant = Literal::string(&to_snake(variant.as_str()));
        self.arms.extend(generate!(Self::#{ variant.clone() } => #wl_variant,));
    }

    fn gens(self) -> impl Iterator<Item = TokenTree> {
        let Self { name, arms } = self;
        generate! {
            impl #name {
                /// Returns the wayland name.
                #[inline]
                pub const fn name(&self) -> &'static str {
                    match self { @arms }
                }
            }
        }
    }
}

struct GenWlEnum {
    name: Ident,
    arms: TokenStream,
}

impl GenWlEnum {
    fn new(name: &Ident) -> Self {
        Self {
            name: name.clone(),
            arms: TokenStream::new(),
        }
    }

    fn add_field(&mut self, variant: &Ident, lit: TokenStream) {
        self.arms.extend(generate!(@lit => Some(Self::#{ variant.clone() }),));
    }

    fn gens(self) -> impl Iterator<Item = TokenTree> {
        let Self { name, arms } = self;
        generate! {
            impl WlEnum for #name {
                #[inline]
                fn from_u32(uint: u32) -> Option<Self> {
                    match uint {
                        @arms
                        _ => None,
                    }
                }

                #[inline]
                fn to_u32(self) -> u32 {
                    self as u32
                }
            }
        }
    }
}
