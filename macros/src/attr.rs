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

// ===== NamedSequence =====

/// `#[ident(values)]`.
pub struct SequenceAttr {
    pub ident: Ident,
    pub parser: Parser,
}

impl SequenceAttr {
    fn parse_attrs_inner(name: &str, parser: &mut Parser) -> Result<Option<Self>, Error> {
        while let Some(_hash) = parser.next_punct_of('#') {
            let _style = parser.next_punct_of('!');
            let group = parser.group_of(Delimiter::Bracket)?;
            let mut input = group.body_parser();
            let ident = input.parse::<Ident>()?;
            if ident.as_str() != name {
                continue;
            }
            let input = input.group_of(Delimiter::Parenthesis)?;
            return Ok(Some(Self {
                ident,
                parser: input.body_parser(),
            }));
        }
        Ok(None)
    }

    /// This will drain and filter all attributes with ident of `name`.
    pub fn parse_attrs(name: &str, parser: &mut Parser) -> Result<Self, Error> {
        match Self::parse_attrs_inner(name, parser)? {
            Some(ok) => Ok(ok),
            None => Err(Error::spanned(
                format!("attribute `{name}` required"),
                parser.span(),
            )),
        }
    }

    /// This will drain and filter all attributes with ident of `name`.
    ///
    /// If there is no specified attribute, returned sequence will be empty.
    pub fn parse_attrs_opt(name: &str, parser: &mut Parser) -> Result<Self, Error> {
        match Self::parse_attrs_inner(name, parser)? {
            Some(ok) => Ok(ok),
            None => Ok(Self {
                ident: Ident::new(name, Span::call_site()),
                parser: Parser::new(TokenStream::new()),
            }),
        }
    }

    pub fn next_flag_of(&mut self, flag: &str) -> Result<bool, Error> {
        let ok = self.parser.next_if_map(|tree| match tree {
            TokenTree::Ident(id) if id.as_str() == flag => Ok(id),
            tree => Err(tree),
        });
        if ok.is_some() {
            self.check_leftover()?;
        }
        Ok(ok.is_some())
    }

    pub fn try_next_named<T: Parse>(&mut self) -> Result<(Ident, T), Error> {
        let ident = self.parser.ident()?;
        self.parser.punct_of('=')?;
        let token = self.parser.parse()?;
        self.check_leftover()?;
        Ok((ident, token))
    }

    pub fn next_named_of<T: Parse>(&mut self, name: &str) -> Result<Option<T>, Error> {
        let peek = self.parser.next_if_map(|tree| match tree {
            TokenTree::Ident(id) if id.as_str() == name => Ok(id),
            tree => Err(tree)
        });
        let Some(_) = peek else {
            return Ok(None);
        };
        self.parser.punct_of('=')?;
        let token = self.parser.parse().map(Some)?;
        self.check_leftover()?;
        Ok(token)
    }

    fn check_leftover(&mut self) -> Result<(), Error> {
        match self.parser.next() {
            Some(t) => match t {
                TokenTree::Punct(p) if p.as_char() == ',' => Ok(()),
                t => Err(Error::spanned("unexpected leftover token", t.span())),
            }
            None => Ok(()),
        }
    }

    pub fn check_empty(mut self) -> Result<(), Error> {
        match self.parser.next() {
            None => Ok(()),
            // the span is terrible here
            Some(t) => Err(Error::spanned("unexpected attr", t.span())),
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
        let meta = input.parse_full()?;
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
