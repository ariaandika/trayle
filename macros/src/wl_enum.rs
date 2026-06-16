use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let mut body = Parser::new(parser.group_of(Delimiter::Brace)?.stream().into());

    let mut names_arm = TokenStream::new();
    let mut match_arm = TokenStream::new();
    let mut i = 0u32;

    loop {
        let _ = body.parse::<Attributes>()?;
        let Some(variant) = body.next_ident() else {
            break;
        };
        let lit = if body.next_punct_of('=').is_some() {
            body.lit()?
        } else {
            let lit = Literal::u32_unsuffixed(i);
            i += 1;
            lit
        };

        let wl_variant = Literal::string(&to_snake(&variant.to_string()));

        body.next_punct_of(',');
        match_arm.extend(generate!(#lit => Some(Self::#&variant),));
        names_arm.extend(generate!(Self::#variant => #wl_variant,));
    }

    Ok(generate! {
        impl #&name {
            /// Returns the wayland name.
            #[inline]
            pub const fn name(&self) -> &'static str {
                match self {
                    #names_arm
                }
            }
        }
        impl WlEnum for #&name {
            #[inline]
            fn from_u32(uint: u32) -> Option<Self> {
                match uint {
                    #match_arm
                    _ => None,
                }
            }

            #[inline]
            fn to_u32(self) -> u32 {
                self as u32
            }
        }
        impl std::fmt::Display for #&name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }
        impl display::Display2 for #&name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                std::fmt::Display::fmt(self, f)
            }
        }
    })
}
