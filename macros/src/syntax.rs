#![allow(dead_code)]

use crate::tree::*;
use crate::parser::*;
use crate::codegen::*;
use crate::error::*;
use crate::attr::*;

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        parser.ident()
    }
}

impl Parse for Literal {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        parser.lit()
    }
}

// ===== Vis =====

#[derive(Clone)]
pub enum Vis {
    Inherit,
    Public(Span, Option<Group>),
}

impl Parse for Vis {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        match parser.next_ident_of("pub") {
            Some(id) => Ok(Vis::Public(
                id.span(),
                parser.next_group_of(Delimiter::Parenthesis),
            )),
            None => Ok(Self::Inherit),
        }
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

// ===== Lifetimes =====

#[derive(Clone)]
pub struct Lifetimes {
    pub delim: (Punct, Punct),
    pub lfs: Vec<Lifetime>,
}

impl Parse for Lifetimes {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        let Some(d1) = parser.next_punct_of('<') else {
            return Ok(Self {
                delim: (Punct::new('<', Spacing::Alone), Punct::new('>', Spacing::Alone)),
                lfs: Vec::new(),
            })
        };
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

mod abomination {
    use super::*;
    use std::array::IntoIter as AI;
    use std::iter::{Chain, FlatMap, Flatten};
    use std::option::IntoIter as OI;
    use std::vec::IntoIter as VI;
    impl IntoIterator for Lifetimes {
        type Item = TokenTree;

        type IntoIter = Flatten<OI<Chain<
            Chain<
                OI<TokenTree>,
                FlatMap<
                    VI<Lifetime>,
                    Chain<AI<TokenTree, 2>, OI<TokenTree>>,
                    fn(Lifetime) -> Chain<AI<TokenTree, 2>, OI<TokenTree>>,
                >,
            >,
            OI<TokenTree>,
        >>>;

        fn into_iter(self) -> Self::IntoIter {
            if self.lfs.is_empty() {
                return None.into_iter().flatten();
            }
            fn mapme(e: Lifetime) -> Chain<AI<TokenTree, 2>, OI<TokenTree>> {
                e.into_iter()
                    .chain(Some(TokenTree::Punct(Punct::new(',', Spacing::Alone))))
            }

            Some(Some(TokenTree::Punct(self.delim.0))
                .into_iter()
                .chain(self.lfs.into_iter().flat_map(mapme as _))
                .chain(Some(TokenTree::Punct(self.delim.1)))).into_iter().flatten()
        }
    }
}

// ===== Enum =====

#[derive(Clone)]
pub struct EnumItem {
    pub attrs: Attributes,
    pub vis: Vis,
    pub enum_kw: Ident,
    pub name: Ident,
    pub body: Group,
}

impl Parse for EnumItem {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        Ok(Self {
            attrs: parser.parse()?,
            vis: parser.parse()?,
            enum_kw: parser.ident_of("enum")?,
            name: parser.ident()?,
            body: parser.group_of(Delimiter::Brace)?,
        })
    }
}

impl EnumItem {
    pub fn variant(parser: &mut Parser) -> Result<Option<Variant>, Error> {
        if parser.peek().is_none() {
            return Ok(None);
        }
        parser.parse().map(Some)
    }
}

// ===== FieldNamed =====

#[derive(Clone)]
pub struct FieldNamed {
    pub attrs: Attributes,
    pub vis: Vis,
    pub ident: Ident,
    pub col: Punct,
    pub ty: TokenStream,
}

impl Parse for FieldNamed {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        Ok(Self {
            attrs: parser.parse()?,
            vis: parser.parse()?,
            ident: parser.ident()?,
            col: parser.punct_of(':')?,
            ty: {
                // the "anonymus" way of parsing rust type, with comma or eof delimited
                // - but comma can also appear in the middle of type
                // - although its always appear inside group, including `<>` group
                // - but `<>` does not captured as `Group` token tree
                // - thus manual depth tracking is needed
                // - but `->` can appear in fn type and is not closing delimiter
                // - thus spacing joint tracking is also needed
                // - but `<<` and `>>` is spacing joint, while can be group delimiter
                // - thus one cannot simply do joint tracking
                //
                // currenty, only `->` tracking is used here
                let mut ty = TokenStream::new();
                let mut depth = 0u32;
                let mut may_arrow = false;
                loop {
                    let tree = parser.next_if_map(|tree| match tree {
                        TokenTree::Punct(p) => {
                            use Spacing as S;
                            match p.as_char() {
                                ',' => if depth == 0 {
                                    return Err(p.into())
                                },
                                '<' => depth += (!may_arrow) as u32,
                                '>' => depth = depth.strict_sub((!may_arrow) as u32),
                                _ => {}
                            }
                            may_arrow = matches!((p.as_char(), p.spacing()), ('-', S::Joint));
                            Ok(p.into())
                        },
                        tree => {
                            may_arrow = false;
                            Ok(tree)
                        },
                    });
                    match tree {
                        Some(tree) => ty.push(tree),
                        None => break
                    }
                }
                ty
            }
        })
    }
}

// ===== Enum =====

#[derive(Clone)]
pub struct Variant {
    pub attrs: Attributes,
    pub ident: Ident,
    pub discr: Option<Discriminant>,
}

#[derive(Clone)]
pub struct Discriminant {
    pub eq: Punct,
    pub expr: TokenStream,
}

impl Parse for Variant {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        Ok(Self {
            attrs: parser.parse()?,
            ident: parser.ident()?,
            discr: {
                let discr = match parser.is_punct_of('=') {
                    Some(_) => Some(parser.parse()?),
                    None => None,
                };
                let _ = parser.next_punct_of(',');
                discr
            }
        })
    }
}

impl Parse for Discriminant {
    fn parse(parser: &mut Parser) -> Result<Self, Error> {
        Ok(Self {
            eq: parser.punct_of('=')?,
            expr: {
                let mut expr = TokenStream::new();
                while parser.is_punct_of(',').is_none() {
                    expr.push(parser.try_next()?);
                }
                expr
            }
        })
    }
}
