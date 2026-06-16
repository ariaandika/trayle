use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.next_ident_of("pub");
    let _ = parser.ident_of("enum")?;
    let name = parser.ident()?;

    let mut body = parser.group_of(Delimiter::Brace)?.body_parser();

    let mut i = 0u32;
    let mut has_custom = None;

    let mut name_getter = GenNameGetter::new(&name);
    let mut wl_enum = GenWlEnum::new(&name);

    loop {
        let _ = body.parse::<Attributes>()?;
        let Some(variant) = body.next_ident() else {
            break;
        };
        let lit = if body.next_punct_of('=').is_some() {
            let lit = body.lit()?;
            if has_custom.is_none() {
                has_custom = Some(lit.span());
            }
            lit
        } else {
            let lit = Literal::u32_unsuffixed(i);
            i += 1;
            lit
        };
        body.next_punct_of(',');

        wl_enum.add_field(&variant, lit);
        name_getter.add_field(&variant);
    }

    if let Some(custom_lit) = has_custom
        && i != 0
    {
        return Err(Error::new(
            "custom value must be all variant or nothing".into(),
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

    fn add_field(&mut self, variant: &Ident, lit: Literal) {
        self.arms.extend(generate!(#lit => Some(Self::#{ variant.clone() }),));
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
