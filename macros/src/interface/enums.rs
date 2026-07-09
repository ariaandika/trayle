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
    is_error: bool,
    name: Ident,
    variants: Vec<Variant>,
}

struct Variant {
    doc: Option<Literal>,
    variant: Ident,
    wl_string: Literal,
    const_name: Ident,
    disc: Literal,
}

impl Parse for Enum {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut is_bitfield = false;
        let mut is_error = false;

        if let Some(mut parser) = attr(parser)? {
            if parser.next_ident_of("bitfield").is_some() {
                parser.next_punct_of(',');
                is_bitfield = true;
            }
            if parser.next_ident_of("error").is_some() {
                parser.next_punct_of(',');
                is_error = true;
            }
            parser.check_empty()?;
        }

        parser.next_ident_of("pub");
        parser.ident_of("enum")?;
        let name = parser.parse()?;

        let mut variants = vec![];
        let mut parser = parser.group_of(Delimiter::Brace)?.body_parser();

        loop {
            let doc = match parser.call(attr)? {
                Some(mut parser) => {
                    parser.ident_of("doc")?;
                    parser.punct_of('=')?;
                    Some(parser.parse_full()?)
                }
                None => None,
            };
            let Some(wl_variant) = parser.next_token::<Ident>() else {
                break;
            };

            if is_error && doc.is_none() {
                return Err(Error::new("`error` enum must have doc comment", wl_variant));
            }

            parser.punct_of('=')?;
            let disc = parser.parse()?;
            parser.next_punct_of(',');

            let variant = wl_variant.to_camel();
            let const_name =
                Ident::new_string(wl_variant.as_str().to_uppercase(), Span::call_site());
            let wl_string = variant.to_lit_snake();

            variants.push(Variant {
                doc,
                variant,
                wl_string,
                const_name,
                disc,
            });
        }

        if is_bitfield && is_error {
            return Err(Error::new("bitfield cannot be an error", name))
        }

        Ok(Self {
            is_bitfield,
            is_error,
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
            let Variant { variant, wl_string, .. } = v;
            g!(Self::#variant => #wl_string,)
        });
        let variants_def = variants.iter().flat_map(|v| {
            let Variant { variant, disc, .. } = v;
            g!(#variant = #disc,)
        });
        let from_arms = variants.iter().flat_map(|v|{
            let Variant { variant, disc, .. } = v;
            g!(#disc => Some(Self::#variant),)
        });

        let messages = self.is_error.then_stream(||{
            let msgs = variants.iter().flat_map(|v| {
                let Variant { doc, variant, .. } = v;
                let mut doc = doc.as_ref().expect("error are asserted to have doc").to_string();
                doc.make_ascii_lowercase();
                let message = Literal::string(doc.trim_start().trim_end_matches('.'));
                g!(Self::#variant => #message,)
            });
            g! {
                #[inline]
                pub const fn message(&self) -> &'static str {
                    match self { @msgs }
                }
            }
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

                @messages
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

            impl FieldDisplay for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    f.write_str(self.name())
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

        let open = Literal::string("<");
        let close = Literal::string(">");
        let sepr = Literal::string("|");
        let fmt = variants.iter().flat_map(|v|{
            let Variant { wl_string, disc, .. } = v;
            g! {
                if self.0 & #disc == #disc {
                    f.write_str(prefix)?;
                    prefix = #sepr;
                    f.write_str(#wl_string)?;
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
                    FieldDisplay::fmt(self, f)
                }
            }

            impl FieldDisplay for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    if self.0 == 0 {
                        f.write_str("<none>")
                    } else {
                        let mut prefix = #open;
                        @fmt
                        f.write_str(#close)
                    }
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
                Self(self.0 #op rhs.0)
            }
        }
    }
}
