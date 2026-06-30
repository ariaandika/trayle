use crate::tree::*;
use crate::error::*;
use crate::parser::*;

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

pub fn attrs_anon(parser: &mut Parser) -> Result<TokenStream> {
    let mut attrs = TokenStream::new();
    while let Some(punct) = parser.next_punct_of('#') {
        attrs.push(punct);
        attrs.push(parser.group_of(Delimiter::Bracket)?);
    }
    Ok(attrs)
}
