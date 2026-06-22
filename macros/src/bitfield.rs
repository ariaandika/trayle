use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let name = parser.ident()?;
    let _ = parser.punct_of(';')?;

    let mut consts = GenBitflag::default();
    let mut display = GenDisplay::default();

    while let Some(Variant { attrs, ident, discr, .. }) = parser.punctuated(',')? {
        let Some(Discriminant { expr, .. }) = discr else {
            return Err(Error::spanned("bitfield require explicit discriminant", ident.span()));
        };
        let wl_name = to_snake(&ident.to_string());
        consts.add_variant(&wl_name, attrs, &expr);
        display.add_variant(&wl_name, &expr);
    }

    let wl_enum = generate! {
        impl WlEnum for #name {
            #[inline]
            fn from_u32(uint: u32) -> Option<Self> {
                Some(Self(uint))
            }
            #[inline]
            fn to_u32(self) -> u32 {
                self.#ZERO
            }
        }
    };

    Ok(wl_enum
        .chain(consts.generate(&name))
        .chain(display.generate(&name))
        .chain(gen_bitops(&name, "BitAnd", "bitand", '&'))
        .chain(gen_bitops(&name, "BitOr", "bitor", '|'))
        .chain(gen_bitops(&name, "BitXor", "bitxor", '^'))
        .collect())
}

fn gen_bitops(name: &Ident, trait_: &str, fn_: &str, op: char) -> impl Iterator<Item = TokenTree> {
    let tr = Ident::new(trait_, Span::call_site());
    let f = Ident::new(fn_, Span::call_site());
    let op = Punct::new(op, Spacing::Alone);
    generate! {
        impl std::ops::#tr for #{name.clone()} {
            type Output = Self;
            #[inline]
            fn #f(self, rhs: Self) -> Self::Output {
                Self(self.#ZERO #op rhs.#ZERO)
            }
        }
    }
}

// ===== impls =====

#[derive(Default)]
struct GenBitflag {
    has_none: bool,
    consts: TokenStream,
}

impl GenBitflag {
    fn add_variant(&mut self, wl_name: &str, attrs: Attributes, expr: &TokenStream) {
        let const_entry = Ident::new_string(wl_name.to_uppercase(), Span::call_site());
        if const_entry.as_str() == "NONE" {
            self.has_none = true;
        }
        self.consts.extend(generate! {
            @attrs
            pub const #const_entry: Self = Self(@expr);
        });
    }

    fn generate(self, name: &Ident) -> impl Iterator<Item = TokenTree> {
        let Self { has_none, consts } = self;
        let none_const = stream_if(!has_none, || {
            generate! {
                pub const NONE: Self = Self(#ZERO);
            }
        });
        generate! {
            impl crate::bitflags::Flags for #name {
                fn bits(self) -> u32 {
                    self.#ZERO
                }
            }
            impl #name {
                @consts
                @none_const
            }
        }
    }
}

#[derive(Default)]
struct GenDisplay {
    tokens: TokenStream,
}

impl GenDisplay {
    fn add_variant(&mut self, wl_name: &str, expr: &TokenStream) {
        let wl_name = Literal::string(wl_name);
        let sepr = Literal::character('|');

        self.tokens.extend(generate! {
            if self.#ZERO & @expr == @expr {
                sepr.fmt(f)?;
                sepr = #sepr;
                #wl_name.fmt(f)?;
            }
        });
    }

    fn generate(self, name: &Ident) -> impl Iterator<Item = TokenTree> {
        let Self { tokens } = self;
        let lt = Literal::character('<');
        let gt = Literal::character('>');

        generate! {
            impl std::fmt::Display for #name {
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    if self.#ZERO == #ZERO {
                        "<none>".fmt(f)?;
                    } else {
                        let mut sepr = #lt;
                        @tokens
                        #gt.fmt(f)?;
                    }
                    Ok(())
                }
            }
            impl display::Display2 for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    std::fmt::Display::fmt(self, f)
                }
            }
        }
    }
}
