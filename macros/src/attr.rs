use crate::tree::*;
use crate::parser::*;
use crate::codegen::*;
use crate::error::*;

// ===== Meta =====

#[derive(Clone)]
pub enum Meta {
    None,
    Seq(Group),
    Expr(Punct, TokenStream),
}

impl Parse for Meta {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let Some(tree) = parser.next() else {
            return Ok(Self::None);
        };
        match tree {
            TokenTree::Group(g) if g.delimiter() == Delimiter::Parenthesis => Ok(Self::Seq(g)),
            TokenTree::Punct(p) if p.as_char() == '=' => Ok(Self::Expr(p, parser.drain())),
            tree => Err(Error::spanned("unexpected token, expected `(..)`, `=` or nothing", tree.span())),
        }
    }
}

impl ToTokens for Meta {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Meta::None => {}
            Meta::Seq(g) => g.to_tokens(tokens),
            Meta::Expr(p, t) => {
                p.to_tokens(tokens);
                t.to_tokens(tokens);
            }
        }
    }
}

// ===== Attribute =====

/// Outer attribute.
#[derive(Clone)]
pub struct Attribute {
    pub hash: Punct,
    pub style: Option<Punct>,
    pub delim: Delimiter,
    /// Actually, this can be path
    pub ident: Ident,
    pub meta: Meta,
}

impl Parse for Attribute {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let hash = parser.punct_of('#')?;
        let style = parser.next_punct_of('!');
        let group = parser.group_of(Delimiter::Bracket)?;
        let delim = group.delimiter();
        let mut input = Parser::new(group.stream());
        let ident = input.parse()?;
        let meta = input.parse()?;
        Ok(Self { hash, style, delim, ident, meta })
    }
}

impl ToTokens for Attribute {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.hash.to_tokens(tokens);
        if let Some(p) = &self.style {
            p.to_tokens(tokens);
        }
        Group::new(self.delim, {
            let mut tokens = self.ident.to_token_stream();
            self.meta.to_tokens(&mut tokens);
            tokens
        })
        .to_tokens(tokens);
    }
}

impl IntoIterator for Attribute {
    type Item = TokenTree;

    type IntoIter = IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        let mut tokens = TokenStream::new();
        tokens.push(self.hash);
        if let Some(punct) = self.style {
            tokens.push(punct);
        }
        let mut input = TokenStream::new();
        input.push(self.ident);
        tokens.push(Group::new(
            self.delim,
            match self.meta {
                Meta::None => input,
                Meta::Seq(g) => {
                    input.push(g);
                    input
                }
                Meta::Expr(punct, tokens) => {
                    input.push(punct);
                    input.extend(tokens);
                    input
                },
            },
        ));
        tokens.into_iter()
    }
}

// ===== Attributes =====

#[derive(Clone)]
pub struct Attributes {
    pub attrs: Vec<Attribute>,
}

impl Attributes {
    pub fn find_seq_with<T: Parse, F: Fn(&str) -> bool>(&self, f: F) -> Result<Option<(Ident, T)>, Error> {
        for attr in &self.attrs {
            if !f(attr.ident.as_str()) {
                continue;
            }
            let Meta::Seq(seq) = &attr.meta else {
                continue;
            };
            return Ok(Some((attr.ident.clone(), Parser::new(seq.stream()).parse()?)));
        }
        Ok(None)
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
