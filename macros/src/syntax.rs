use proc_macro::*;

use Delimiter as Delim;

use crate::Error;
use crate::codegen::ToTokens;
use crate::parser::{Parse, Parser};

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
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend([self.backtick.into(), self.name.into()] as [TokenTree; _]);
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
    fn into_tokens(self, tokens: &mut TokenStream) {
        use std::iter::once;
        tokens.extend(once(self.delim.0));
        for lf in self.lfs {
            lf.into_tokens(tokens);
            crate::codegen::gen_token!(,).into_tokens(tokens);
        }
        tokens.extend(once(self.delim.1));
    }
}

// ===== Attribute =====

#[allow(unused)]
pub struct Attribute {
    pub hash: Punct,
    pub delim: Delim,
    pub tokens: TokenStream,
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let hash = parser.punct_of('#')?;
        let group = parser.group_of(Delim::Bracket)?;
        Ok(Self {
            hash,
            delim: group.delimiter(),
            tokens: group.stream(),
        })
    }
}

pub struct Attributes {
    pub attrs: Vec<Attribute>,
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

