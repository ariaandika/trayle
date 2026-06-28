use crate::prelude::*;
use crate::interface::{Op, OpKind};

pub struct Ops {
    pub kind: OpKind,
    pub ops: Vec<Op>,
}

impl Ops {
    fn empty(kind: OpKind) -> Self {
        Self { kind, ops: vec![] }
    }

    pub fn parse(kind: OpKind, parser: &mut Parser) -> Result<Self, Error> {
        let name = match kind {
            OpKind::Request => "request",
            OpKind::Event => "event",
        };
        let Some(TokenTree::Ident(id)) = parser.peek2() else {
            return Ok(Self::empty(kind));
        };
        if !id.as_str().eq_ignore_ascii_case(name) {
            return Ok(Self::empty(kind));
        }
        let mut ops = Vec::with_capacity(4);
        parser.ident_of("impl")?;
        parser.ident()?;
        let mut body_parser = parser.group_of(Delimiter::Brace)?.body_parser();
        let mut version = 1;
        while body_parser.peek().is_some() {
            let mut op = body_parser.parse::<Op>()?;
            match op.since {
                Some(since) => {
                    if since > version {
                        version = since;
                    } else {
                        let m = if since == version { "equal" } else { "less" };
                        return Err(Error::spanned(
                            format!("version is {m} than previous op"),
                            op.name.span(),
                        ));
                    }
                }
                None => op.since = Some(version),
            }
            ops.push(op);
        }
        Ok(Self { kind, ops })
    }
}

impl std::ops::Deref for Ops {
    type Target = Vec<Op>;

    fn deref(&self) -> &Self::Target {
        &self.ops
    }
}
