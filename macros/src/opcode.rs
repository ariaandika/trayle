use crate::prelude::*;

fn err_empty_enum() -> Error {
    // Perhaps allowed for enum that needs to exists for trait implementation ?
    Error::new("empty enum is not supported".into(), Span::call_site())
}

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let wl_name = Literal::string(&to_snake(&name.to_string()));
    let mut body = Parser::new(parser.group_of(Delimiter::Brace)?.stream());

    let zero = Literal::u8_unsuffixed(0);
    let mut last_variant = body.next_ident().ok_or_else(err_empty_enum)?;
    let mut i = 1;

    loop {
        body.next_punct_of(',');
        let Some(next_ident) = body.next_ident() else {
            break;
        };
        last_variant = next_ident;
        i += 1;
    }

    let from_op = match i {
        1 => generate! {
            if op == #zero { Some(Self::#last_variant) } else { None }
        },
        _ => generate! {
            if op as u8 <= Self::#last_variant as u8 {
                Some(unsafe { std::mem::transmute::<u8, Self>(op as u8) })
            } else {
                None
            }
        },
    };

    Ok(generate! {
        impl OpCode for #name {
            const OPNAME: &str = #wl_name;

            #[inline]
            fn from_op(op: u16) -> Option<Self> {
                #from_op
            }

            #[inline]
            fn to_op(self) -> u16 {
                self as u16
            }
        }
    })
}
