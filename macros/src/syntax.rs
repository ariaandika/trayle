use crate::tree::*;
use crate::parser::*;
use crate::codegen::*;
use crate::error::*;

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        parser.ident()
    }
}

// ===== Lifetime =====

#[derive(Clone)]
pub struct Lifetime {
    pub backtick: Punct,
    pub name: Ident,
}

impl Parse for Lifetime {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        Ok(Self {
            backtick: parser.punct_of('\'')?,
            name: parser.ident()?,
        })
    }
}

impl ToTokens for Lifetime {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend([TokenTree::from(self.backtick.clone()), self.name.clone().into()]);
    }
}

impl IntoIterator for Lifetime {
    type Item = TokenTree;

    type IntoIter = std::array::IntoIter<TokenTree, 2>;

    fn into_iter(self) -> Self::IntoIter {
        [TokenTree::from(self.backtick), self.name.into()].into_iter()
    }
}

#[derive(Clone)]
pub struct Lifetimes {
    pub delim: (Punct, Punct),
    pub lfs: Vec<Lifetime>,
}

impl Lifetimes {
    pub fn parse_opt(parser: &mut Parser) -> Result<Option<Self>, Error> {
        parser.is_punct_of('<').map(|()|parser.parse()).transpose()
    }
}

impl Parse for Lifetimes {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let d1 = parser.punct_of('<')?;
        let mut lfs = vec![];
        let d2 = loop {
            match parser.next_punct_of('>') {
                Some(d2) => break d2,
                None => {
                    lfs.push(parser.parse()?);
                    parser.next_punct_of(',');
                },
            }
        };
        Ok(Self {
            delim: (d1, d2),
            lfs,
        })
    }
}

impl ToTokens for Lifetimes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if self.lfs.is_empty() {
            return;
        }
        tokens.push(self.delim.0.clone());
        for lf in &self.lfs {
            lf.to_tokens(tokens);
            Punct::new(',', Spacing::Alone).to_tokens(tokens);
        }
        tokens.push(self.delim.1.clone());
    }
}

mod abomination {
    use super::*;
    use std::array::IntoIter as AI;
    use std::iter::{Chain, FlatMap};
    use std::option::IntoIter as OI;
    use std::vec::IntoIter as VI;
    impl IntoIterator for Lifetimes {
        type Item = TokenTree;

        type IntoIter = Chain<
            Chain<
                OI<TokenTree>,
                FlatMap<
                    VI<Lifetime>,
                    Chain<AI<TokenTree, 2>, OI<TokenTree>>,
                    fn(Lifetime) -> Chain<AI<TokenTree, 2>, OI<TokenTree>>,
                >,
            >,
            OI<TokenTree>,
        >;

        fn into_iter(self) -> Self::IntoIter {
            fn mapme(e: Lifetime) -> Chain<AI<TokenTree, 2>, OI<TokenTree>> {
                e.into_iter()
                    .chain(Some(TokenTree::Punct(Punct::new(',', Spacing::Alone))))
            }

            Some(TokenTree::Punct(self.delim.0))
                .into_iter()
                .chain(self.lfs.into_iter().flat_map(mapme as _))
                .chain(Some(TokenTree::Punct(self.delim.1)))
        }
    }
}

// ===== Attribute =====

#[derive(Clone)]
pub struct Attribute {
    pub hash: Punct,
    pub delim: Delimiter,
    pub tokens: TokenStream,
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let hash = parser.punct_of('#')?;
        let group = parser.group_of(Delimiter::Bracket)?;
        Ok(Self {
            hash,
            delim: group.delimiter(),
            tokens: group.stream(),
        })
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.hash.to_tokens(tokens);
        Group::new(self.delim, self.tokens.clone()).to_tokens(tokens);
    }
}

impl IntoIterator for Attribute {
    type Item = TokenTree;

    type IntoIter = std::array::IntoIter<TokenTree, 2>;

    fn into_iter(self) -> Self::IntoIter {
        [TokenTree::from(self.hash), Group::new(self.delim, self.tokens).into()].into_iter()
    }
}

#[derive(Clone)]
pub struct Attributes {
    pub attrs: Vec<Attribute>,
}

impl Attributes {
    pub fn attrs_parser(self) -> impl Iterator<Item = Parser> {
        self.attrs.into_iter().map(|e| Parser::new(e.tokens))
    }
}

impl Parse for Attributes {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut attrs = vec![];
        while parser.is_punct_of('#').is_some() {
            attrs.push(parser.parse()?);
        }
        Ok(Self { attrs })
    }
}

impl ToTokens for Attributes {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        for attr in &self.attrs {
            attr.to_tokens(tokens);
        }
    }
}

impl IntoIterator for Attributes {
    type Item = TokenTree;

    type IntoIter = std::iter::Flatten<std::vec::IntoIter<Attribute>>;

    fn into_iter(self) -> Self::IntoIter {
        self.attrs.into_iter().flatten()
    }
}
