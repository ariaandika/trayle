use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let mod_attrs = parser.parse::<Attributes>()?;
    let mod_vis = parser.ident_of("pub")?;
    let mod_kw = parser.ident_of("mod")?;
    let mod_name = parser.ident()?;
    let _ = parser.punct_of(';')?;

    let attrs = parser.parse::<Attributes>()?;
    let vis = parser.ident_of("pub")?;
    let enum_kw = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let _ = parser.punct_of(';')?;

    let mut mod_declare = TokenStream::new();
    let mut reexports = TokenStream::new();
    let mut variants = TokenStream::new();
    let mut names = TokenStream::new();
    let mut len = 0;

    loop {
        let (vis, attr) = if parser.is_punct_of('#').is_some() {
            let attr = parser.parse::<Attribute>()?;
            let vis = parser.ident_of("pub")?;
            (vis, Some(attr))
        } else if let Some(vis) = parser.next_ident_of("pub") {
            (vis, None)
        } else {
            break;
        };
        len += 1;

        let is_todo = attr
            .and_then(|attr| Parser::new(attr.tokens).next_ident_of("todo"))
            .is_some();
        let mod_kw = parser.ident_of("mod")?;
        let name = parser.ident()?;
        let semi = parser.punct_of(';')?;

        let name_string = name.to_string();
        let name_camel = Ident::new(&to_camel(&name_string), Span::call_site());

        variants.extend([
            TokenTree::from(name_camel.clone()),
            Punct::new(',', Spacing::Alone).into(),
        ]);
        names.extend([
            TokenTree::from(Literal::string(&name_string)),
            Punct::new(',', Spacing::Alone).into(),
        ]);

        if !is_todo {
            reexports.extend(generate!(pub use super::#&name as #name_camel;));
            mod_declare.extend([
                TokenTree::from(vis),
                mod_kw.into(),
                name.into(),
                semi.into(),
            ])
        }
    }

    let len = Literal::usize_unsuffixed(len);

    Ok(generate! {
        #attrs
        #vis #enum_kw #&name {
            #variants
        }

        impl #&name {
            #[doc = " Returns lower cased name of current interface."]
            #[inline]
            pub fn name(&self) -> &'static str {
                static LOOKUP: [&'static str; #len] = [#names];
                unsafe { LOOKUP.get_unchecked(*self as usize) }
            }
        }

        impl std::fmt::Display for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }

        #mod_declare

        #mod_attrs
        #mod_vis #mod_kw #mod_name {
            #reexports
        }
    })
}
