use crate::prelude::*;

pub fn impl_interface_id(mut parser: Parser) -> Result<TokenStream, Error> {
    let mod_attrs = parser.call(attrs_anon)?;
    parser.ident_of("pub")?;
    parser.ident_of("mod")?;
    let mod_name = parser.parse::<Ident>()?;
    parser.punct_of(';')?;

    let enum_attrs = parser.call(attrs_anon)?;
    parser.ident_of("pub")?;
    parser.ident_of("enum")?;
    let name = parser.parse::<Ident>()?;
    parser.punct_of(';')?;

    let mut variants = Vec::with_capacity(32);

    while let Some(variant) = parser.next_token::<Ident>() {
        variants.push(variant);
        parser.next_punct_of(',');
    }

    let enum_variants = variants.iter().flat_map(|variant|g!(#variant,));
    let names_len = Literal::usize_unsuffixed(variants.len());
    let names = variants.iter().flat_map(|variant|{
        let wl_variant_str = variant.to_lit_snake();
        g!(#wl_variant_str,)
    });
    let as_interfaces = variants.iter().flat_map(|variant|g! {
        impl AsInterface for #variant {
            #[inline]
            fn interface(&self) -> #name {
                #name::#variant
            }
        }
    });

    let camel_cased_mod = variants.iter().flat_map(|v|{
        // NOTE: assuming the module has already declared
        let v_mod = v.to_snake();
        g!(pub use super::#v_mod as #v;)
    });

    Ok(g! {
        @enum_attrs
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum #name {
            @enum_variants
        }

        impl #name {
            #[doc = " Returns the wayland name."]
            #[inline]
            pub const fn name(self) -> &'static str {
                static LOOKUP: [&'static str; #names_len] = [@names];
                unsafe { *LOOKUP.as_ptr().add(self as usize) }
            }
        }

        impl std::fmt::Display for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }

        @as_interfaces

        @mod_attrs
        pub mod #mod_name {
            @camel_cased_mod
        }
    }.collect())
}
