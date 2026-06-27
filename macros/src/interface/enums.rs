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

pub struct Enum {
    pub is_bitfield: bool,
    pub name: Ident,
    pub variants: Vec<Variant>,
}

pub struct Variant {
    pub variant: Ident,
    pub disc: Literal,
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

        parser.parse::<Vis>()?;
        parser.ident_of("enum")?;
        let name = parser.ident()?;

        let mut variants = vec![];
        let mut parser = parser.group_of(Delimiter::Brace)?.body_parser();
        let mut i = 0;

        while let Some(wl_variant) = parser.next_ident() {
            let disc = if parser.next_punct_of('=').is_some() {
                parser.lit()?
            } else {
                Literal::u32_unsuffixed({
                    let i_ = i;
                    i += 1;
                    i_
                })
            };
            let variant = Ident::new_string(to_camel(wl_variant.as_str()), wl_variant.span());
            variants.push(Variant {
                variant,
                disc,
            });
            parser.next_punct_of(',');
        }

        Ok(Self { is_bitfield, name, variants })
    }
}

impl Enum {
    pub fn gen_enum(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.name;
        if self.is_bitfield {
            Either::Left(g! {
                #[derive(Debug, Clone, Copy)]
                pub struct #name(u32);
            })
        } else {
            let variants = self.variants.iter().flat_map(|v| {
                let var = Ident::new_string(to_camel(v.variant.as_str()), v.variant.span());
                let disc = &v.disc;
                g!(#var = #disc,)
            });
            Either::Right(g! {
                #[derive(Debug, Clone, Copy)]
                pub enum #name {
                    @variants
                }
            })
        }
    }

    pub fn gen_display(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.name;
        let names = stream_if(!self.is_bitfield, ||{
            let names = self.variants.iter().flat_map(|v|{
                let variant = &v.variant;
                let wl_variant = Literal::string(&to_snake(variant.as_str()));
                g!(Self::#variant => #wl_variant,)
            });
            g! {
                impl #name {
                    /// Returns the wayland name.
                    #[inline]
                    pub const fn name(&self) -> &'static str {
                        match self { @names }
                    }
                }
            }
        });
        let display = if self.is_bitfield {
            let lt = Literal::character('<');
            let gt = Literal::character('>');
            let sepr = Literal::character('|');
            let fmt = self.variants.iter().flat_map(|v|{
                let wl_name = Literal::string(&to_snake(v.variant.as_str()));
                let disc = &v.disc;
                g! {
                    if self.#ZERO & #disc == #disc {
                        sepr.fmt(f)?;
                        sepr = #sepr;
                        #wl_name.fmt(f)?;
                    }
                }
            });
            Either::Left(g! {
                if self.#ZERO == #ZERO {
                    "<none>".fmt(f)?;
                } else {
                    let mut sepr = #lt;
                    @fmt
                    #gt.fmt(f)?;
                }
                Ok(())
            })
        } else {
            Either::Right(g!(self.name().fmt(f)))
        };
        g! {
            impl std::fmt::Display for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    @display
                }
            }
            impl display::Display2 for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    std::fmt::Display::fmt(self, f)
                }
            }
        }
        .chain(names)
    }

    pub fn gen_wl_enum(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.name;

        let from = if self.is_bitfield {
            Either::Left(g!(Some(Self(uint))))
        } else {
            let arms = self.variants.iter().flat_map(|v|{
                let Variant { variant, disc, .. } = v;
                g!(#disc => Some(Self::#variant),)
            });
            Either::Right(g!(match uint { @arms _ => None, }))
        };
        let to = if self.is_bitfield {
            Either::Left(g!(self.#ZERO))
        } else {
            Either::Right(g!(self as u32))
        };

        g! {
            impl WlEnum for #name {
                #[inline]
                fn from_u32(uint: u32) -> Option<Self> {
                    @from
                }

                #[inline]
                fn to_u32(self) -> u32 {
                    @to
                }
            }
        }
    }

    // bitfield only

    pub fn gen_consts(&self) -> impl Iterator<Item = TokenTree> {
        stream_if(self.is_bitfield, ||{
            let name = &self.name;
            let consts = self.variants.iter().flat_map(|v|{
                let upname = v.variant.as_str().to_uppercase();
                let upname = Ident::new_string(upname, v.variant.span());
                let disc = &v.disc;
                g!(pub const #upname: Self = Self(#disc);)
            });
            g! {
                impl #name {
                    @consts
                }
            }
        })
    }

    pub fn gen_impl_flags(&self) -> impl Iterator<Item = TokenTree> {
        stream_if(self.is_bitfield, ||{
            let name = &self.name;
            g! {
                impl crate::bitflags::Flags for #name {
                    fn bits(self) -> u32 {
                        self.#ZERO
                    }
                }
            }
        })
    }

    pub fn gen_bit_ops(&self) -> impl Iterator<Item = TokenTree> {
        stream_if(self.is_bitfield, ||{
            let name = &self.name;
            gen_bitops(name, "BitAnd", "bitand", '&')
                .chain(gen_bitops(name, "BitOr", "bitor", '|'))
                .chain(gen_bitops(name, "BitXor", "bitxor", '^'))
        })
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
