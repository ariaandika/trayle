use crate::prelude::*;

pub fn impl_interface_id(mut parser: Parser) -> Result<TokenStream, Error> {
    parser.parse::<Attributes>()?;
    parser.ident_of("pub")?;
    parser.ident_of("enum")?;
    let name = parser.ident()?;
    parser.punct_of(';')?;

    let mut variants = Vec::with_capacity(32);

    while let Some(variant) = parser.next_ident() {
        variants.push(variant);
        parser.next_punct_of(',');
    }

    let as_interfaces = variants.iter().flat_map(|variant|{
        g! {
            impl AsInterface for #variant {
                #[inline]
                fn interface(&self) -> #name {
                    #name::#variant
                }
            }
        }
    });
    let names_len = Literal::usize_unsuffixed(variants.len());
    let names = variants.iter().flat_map(|variant|{
        let wl_variant_str = Literal::string(&to_snake(variant.as_str()));
        g!(#wl_variant_str,)
    });
    let variants = variants.iter().flat_map(|variant|{
        g!(#variant,)
    });

    Ok(g! {
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #name {
            @variants
        }

        impl #name {
            #[doc = " Returns the wayland name."]
            #[inline]
            pub fn name(&self) -> &'static str {
                static LOOKUP: [&'static str; #names_len] = [@names];
                unsafe { LOOKUP.get_unchecked(*self as usize) }
            }
        }

        impl std::fmt::Display for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }

        @as_interfaces
    }.collect())
}
