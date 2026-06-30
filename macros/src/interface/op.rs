use crate::interface::attr;
use crate::prelude::*;
use crate::interface::arg::*;

#[derive(Debug, Clone, Copy)]
pub enum OpKind {
    Request,
    Event,
}

/// `#[destructor, since = 5]`
/// `delete_id(id: uint, parent: object<wl_registry>?)`
pub struct Op {
    pub wl_name: Ident,
    pub op_name: Ident,
    pub op_span: Span,

    pub kind: OpKind,
    pub since: Option<LitInt>,
    pub is_destructor: bool,

    pub lf_ph: LfPh,
    pub fd_idx: Option<usize>,

    pub args: Vec<Arg>,
}

impl Op {
    pub fn parse(kind: OpKind, parser: &mut Parser) -> Result<Self, Error> {
        let mut since = None;
        let mut is_destructor = false;

        if let Some(mut parser) = attr(parser)? {
            if parser.next_ident_of("since").is_some() {
                parser.punct_of('=')?;
                since = Some(parser.parse::<LitInt>()?);
                parser.next_punct_of(',');
            }

            if parser.next_ident_of("destructor").is_some() {
                parser.next_punct_of(',');
                is_destructor = true;
            }

            parser.check_empty()?;
        }

        parser.next_ident_of("pub");
        parser.ident_of("fn")?;

        let mut wl_name = parser.parse::<Ident>()?;
        let op_span = wl_name.unspan();
        let op_name = wl_name.to_camel();

        let mut lf_ph = LfPh::new(false);
        let mut has_new_id = false;
        let mut fd_idx = None;

        let mut args = vec![];
        let mut arg_parser = parser.group_of(Delimiter::Parenthesis)?.body_parser();

        while arg_parser.peek().is_some() {
            let arg = arg_parser.parse::<Arg>()?;

            if arg.is_lf() {
                lf_ph = LfPh::new(true);
            }
            if arg.ty.as_new_id().is_some() {
                if has_new_id {
                    return Err(Error::new("only one new_id is supported", arg.name));
                }
                has_new_id = true;
            }
            if arg.ty.is_fd() {
                if fd_idx.is_some() {
                    return Err(Error::new("only one fd is supported", arg.name));
                }
                fd_idx = Some(args.len());
            }

            args.push(arg);
            arg_parser.next_punct_of(',');
        }

        parser.punct_of(';')?;

        Ok(Self {
            wl_name,
            op_name,
            op_span,
            kind,
            since,
            is_destructor,
            args,
            lf_ph,
            fd_idx,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LfPh {
    has_lf: bool,
    has_name: bool,
}

impl LfPh {
    fn new(has_lf: bool) -> Self {
        Self { has_lf, has_name: false }
    }

    pub fn named(self) -> Self {
        Self {
            has_lf: self.has_lf,
            has_name: true,
        }
    }
}

impl IntoIterator for LfPh {
    type Item = TokenTree;

    type IntoIter = Either<std::option::IntoIter<TokenTree>, std::array::IntoIter<TokenTree, 4>>;

    fn into_iter(self) -> Self::IntoIter {
        if self.has_lf {
            Either::Right(
                [
                    TokenTree::from(Punct::new('<', Spacing::Alone)),
                    TokenTree::from(Punct::new('\'', Spacing::Joint)),
                    TokenTree::from(Ident::new(
                        if self.has_name { "a" } else { "_" },
                        Span::call_site(),
                    )),
                    TokenTree::from(Punct::new('>', Spacing::Alone)),
                ]
                .into_iter(),
            )
        } else {
            Either::Left(None.into_iter())
        }
    }
}
