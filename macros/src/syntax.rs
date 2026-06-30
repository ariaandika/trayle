use crate::span::*;
use crate::tree::*;
use crate::error::*;
use crate::parser::*;

pub fn attr(parser: &mut Parser) -> Result<Option<Parser>, Error> {
    match parser.next_punct_of('#') {
        Some(_) => Ok(Some(parser.group_of(Delimiter::Bracket)?.body_parser())),
        None => Ok(None),
    }
}

pub fn attrs_anon(parser: &mut Parser) -> Result<TokenStream> {
    let mut attrs = TokenStream::new();
    while let Some(punct) = parser.next_punct_of('#') {
        attrs.push(punct);
        attrs.push(parser.group_of(Delimiter::Bracket)?);
    }
    Ok(attrs)
}

#[expect(dead_code)]
pub fn type_anon(parser: &mut Parser) -> TokenStream {
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

// ===== LitInt =====

#[derive(Clone)]
pub struct LitInt {
    int: usize,
    span: Span,
}

impl Parse for LitInt {
    fn parse(parser: &mut Parser) -> Result<Self> {
        let lit = parser.parse::<Literal>()?;
        let int = match lit.to_string().parse::<usize>() {
            Ok(ok) => ok,
            Err(e) => return Err(Error::new(e.to_string(), lit))
        };
        Ok(Self {
            int,
            span: lit.span(),
        })
    }
}

impl LitInt {
    pub fn new(int: usize) -> Self {
        Self {
            int,
            span: Span::call_site(),
        }
    }

    pub fn get(&self) -> usize {
        self.int
    }
}

impl std::fmt::Debug for LitInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.int.fmt(f)
    }
}

impl From<LitInt> for TokenTree {
    fn from(val: LitInt) -> Self {
        TokenTree::Literal(Literal::usize_unsuffixed(val.int)).spanned(val.span)
    }
}
