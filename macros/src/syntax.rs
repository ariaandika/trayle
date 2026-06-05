use proc_macro::*;

use Delimiter as Delim;

use crate::Error;
use crate::parser::{Parse, Parser};

// ===== Attributes =====

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

#[allow(unused)]
pub struct Attributes {
    pub attrs: Vec<Attribute>,
}

impl Parse for Attributes {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let mut attrs = vec![];
        while parser.next_punct_of('#').is_some() {
            attrs.push(parser.parse()?);
        }
        Ok(Self { attrs })
    }
}

