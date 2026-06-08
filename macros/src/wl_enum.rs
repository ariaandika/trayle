use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let mut body = Parser::new(parser.group_of(Delimiter::Brace)?.stream());

    let mut match_arm = TokenStream::new();
    let mut i = 0u32;

    while let Some(variant) = body.next_ident() {
        let lit = if body.next_punct_of('=').is_some() {
            body.lit()?
        } else {
            let lit = Literal::u32_unsuffixed(i);
            i += 1;
            lit
        };
        body.next_punct_of(',');
        match_arm.extend(generate!(#lit => Some(Self::#variant),));
    }

    Ok(generate! {
        impl WlEnum for #name {
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
    })
}
