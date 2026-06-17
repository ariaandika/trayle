use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let EnumItem { name, body, .. } = parser.parse()?;

    let mut i = 0;
    let mut last_variant = None;

    let mut body = body.body_parser();
    let mut names_arm = TokenStream::new();

    while let Some(Variant { ident, discr, .. }) = body.separated(',')? {
        if discr.is_some() {
            return Err(Error::spanned(
                "opcode enum cannot have discriminant",
                ident.span(),
            ));
        }
        let wl_entry = Literal::string(&to_snake(ident.as_str()));
        names_arm.extend(generate!(Self::#ident => #wl_entry,));
        last_variant = Some(ident);
        i += 1;
    }

    let Some(last_variant) = last_variant else {
        return Err(Error::new("opcode enum cannot be empty"));
    };

    let cmp: TokenStream = match i {
        1 => generate!(op == #ZERO).collect(),
        _ => generate!(op as u8 <= Self::#last_variant as u8).collect(),
    };

    let cvt: TokenStream = match i {
        1 => generate!(Self::#last_variant).collect(),
        _ => generate!(unsafe { std::mem::transmute::<u8, Self>(op as u8) }).collect(),
    };

    Ok(generate! {
        impl #name {
            /// Returns the wayland name.
            #[inline]
            pub const fn name(&self) -> &'static str {
                match self {
                    @names_arm
                }
            }
        }
        impl OpCode for #name {
            #[inline]
            fn from_op(op: u16) -> Option<Self> {
                if @cmp { Some(@cvt) } else { None }
            }

            #[inline]
            fn to_op(self) -> u16 {
                self as u16
            }
        }
        impl std::fmt::Display for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }
    }.collect())
}
