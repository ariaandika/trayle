use crate::prelude::*;
use crate::interface::*;

pub struct Enums {
    pub enums: Vec<Enum>,
}

impl Parse for Enums {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut enums = vec![];
        while parser.peek().is_some() {
            enums.push(parser.parse()?);
        }
        Ok(Self { enums })
    }
}

impl Enums {
    pub fn generate(&self) -> impl Iterator<Item = TokenTree> {
        self.enums.iter().flat_map(Enum::generate)
    }
}

pub struct Enum {
    is_bitfield: bool,
    name: Ident,
    variants: Vec<Variant>,
}

struct Variant {
    variant: Ident,
    wl_variant: Literal,
    const_name: Ident,
    disc: Literal,
}

impl Parse for Enum {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut is_bitfield = false;

        if let Some(mut parser) = attr(parser)? {
            if parser.next_ident_of("bitfield").is_some() {
                parser.next_punct_of(',');
                is_bitfield = true;
            }
            parser.check_empty()?;
        }

        parser.next_ident_of("pub");
        parser.ident_of("enum")?;
        let name = parser.parse()?;

        let mut variants = vec![];
        let mut parser = parser.group_of(Delimiter::Brace)?.body_parser();
        let mut i = 0;

        while let Some(wl_variant) = parser.next_token::<Ident>() {
            let disc = if parser.next_punct_of('=').is_some() {
                parser.parse()?
            } else {
                if is_bitfield {
                    return Err(Error::new("bitfield enum must have explicit value", wl_variant))
                }
                let i_ = i;
                i += 1;
                Literal::u32_unsuffixed(i_)
            };

            let variant = wl_variant.to_camel();
            let const_name =
                Ident::new_string(wl_variant.as_str().to_uppercase(), Span::call_site());
            let wl_variant = Literal::string(variant.as_str());

            variants.push(Variant {
                variant,
                wl_variant,
                const_name,
                disc,
            });
            parser.next_punct_of(',');
        }

        Ok(Self {
            is_bitfield,
            name,
            variants,
        })
    }
}

impl Enum {
    fn generate(&self) -> impl Iterator<Item = TokenTree> {
        if self.is_bitfield {
            self.gen_bitfield().left()
        } else {
            self.gen_enum().right()
        }
    }
}

impl Enum {
    fn gen_enum(&self) -> impl Iterator<Item = TokenTree> {
        let Self { name, variants, .. } = self;

        let names = variants.iter().flat_map(|v| {
            let Variant { variant, wl_variant, .. } = v;
            g!(Self::#variant => #wl_variant,)
        });
        let variants_def = variants.iter().flat_map(|v| {
            let Variant { variant, disc, .. } = v;
            g!(#variant = #disc,)
        });
        let from_arms = variants.iter().flat_map(|v|{
            let Variant { variant, disc, .. } = v;
            g!(#disc => Some(Self::#variant),)
        });

        g! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub enum #name {
                @variants_def
            }

            impl #name {
                /// Returns the wayland name.
                #[inline]
                pub const fn name(&self) -> &'static str {
                    match self { @names }
                }
            }

            impl WlEnum for #name {
                #[inline]
                fn from_u32(uint: u32) -> Option<Self> {
                    match uint { @from_arms _ => None, }
                }

                #[inline]
                fn to_u32(self) -> u32 {
                    self as u32
                }
            }

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
        }
    }
}

impl Enum {
    fn gen_bitfield(&self) -> impl Iterator<Item = TokenTree> {
        let Self { name, variants, .. } = self;

        let consts = variants.iter().flat_map(|v|{
            let Variant { const_name, disc, .. } = v;
            g!(pub const #const_name: Self = Self(#disc);)
        });

        let lt = Literal::character('<');
        let gt = Literal::character('>');
        let sepr = Literal::character('|');
        let fmt = variants.iter().flat_map(|v|{
            let Variant { wl_variant, disc, .. } = v;
            g! {
                if self.0 & #disc == #disc {
                    sepr.fmt(f)?;
                    sepr = #sepr;
                    #wl_variant.fmt(f)?;
                }
            }
        });

        g! {
            #[derive(Debug, Clone, Copy, PartialEq, Eq)]
            pub struct #name(u32);

            impl #name {
                @consts
            }

            impl crate::bitflags::Flags for #name {
                #[inline]
                fn bits(self) -> u32 {
                    self.0
                }
            }

            impl WlEnum for #name {
                #[inline]
                fn from_u32(uint: u32) -> Option<Self> {
                    Some(Self(uint))
                }

                #[inline]
                fn to_u32(self) -> u32 {
                    self.0
                }
            }

            impl std::fmt::Display for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    if self.#ZERO == #ZERO {
                        "<none>".fmt(f)?;
                    } else {
                        let mut sepr = #lt;
                        @fmt
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
        .chain(gen_bitops(name, "BitAnd", "bitand", '&'))
        .chain(gen_bitops(name, "BitOr", "bitor", '|'))
        .chain(gen_bitops(name, "BitXor", "bitxor", '^'))
    }
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
