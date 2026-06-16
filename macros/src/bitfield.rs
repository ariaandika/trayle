use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let name = parser.ident()?;
    let _ = parser.punct_of(';')?;

    let zero = Literal::u32_unsuffixed(0);
    let lt = Literal::character('<');
    let gt = Literal::character('>');
    let mut consts = TokenStream::new();
    let mut display = TokenStream::new();

    loop {
        let attrs = parser.parse::<Attributes>()?;
        let Some(entry) = parser.next_ident() else {
            break;
        };
        let _ = parser.punct_of('=')?;
        let value = parser.lit()?;
        let _ = parser.next_punct_of(',');

        let wl_entry = to_snake(&entry.to_string());
        let const_entry = Ident::new(&wl_entry.to_uppercase(), Span::call_site());

        consts.extend(generate! {
            @attrs
            pub const #const_entry: Self = Self(#value);
        });

        let wl_entry = Literal::string(&wl_entry);
        let sepr = Literal::character('|');
        display.extend(generate! {
            if self.#zero & #value == #value {
                sepr.fmt(f)?;
                sepr = #sepr;
                #wl_entry.fmt(f)?;
            }
        });
    }

    Ok(generate! {
        impl std::ops::BitAnd for #name {
            type Output = Self;
            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.#zero & rhs.#zero)
            }
        }
        impl std::ops::BitOr for #name {
            type Output = Self;
            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.#zero | rhs.#zero)
            }
        }
        impl std::ops::BitXor for #name {
            type Output = Self;
            #[inline]
            fn bitxor(self, rhs: Self) -> Self::Output {
                Self(self.#zero ^ rhs.#zero)
            }
        }
        impl WlEnum for #name {
            #[inline]
            fn from_u32(uint: u32) -> Option<Self> {
                Some(Self(uint))
            }
            #[inline]
            fn to_u32(self) -> u32 {
                self.#zero
            }
        }
        impl #name {
            @consts
        }
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                if self.#zero == #zero {
                    "<none>".fmt(f)?;
                } else {
                    let mut sepr = #lt;
                    @display
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
    }.collect())
}
