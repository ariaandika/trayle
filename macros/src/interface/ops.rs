use crate::prelude::*;
use crate::interface::{Op, OpKind};

pub struct Ops {
    pub kind: OpKind,
    pub ops: Vec<Op>,
}

impl Ops {
    pub fn parse(kind: OpKind, parser: &mut Parser) -> Result<Self, Error> {
        let name = match kind {
            OpKind::Request => "request",
            OpKind::Event => "event",
        };
        let Some(TokenTree::Ident(id)) = parser.peek2() else {
            return Ok(Self { kind, ops: vec![] });
        };
        if !id.as_str().eq_ignore_ascii_case(name) {
            return Ok(Self { kind, ops: vec![] });
        }
        let mut ops = Vec::with_capacity(4);
        parser.ident_of("impl")?;
        parser.parse::<Ident>()?;
        let mut body_parser = parser.group_of(Delimiter::Brace)?.body_parser();
        let mut version = 1;
        while body_parser.peek().is_some() {
            let mut op = body_parser.call(|p|Op::parse(kind, p))?;
            match op.since.as_ref() {
                Some(since) => {
                    if since.get() > version {
                        version = since.get();
                    } else {
                        let m = if since.get() == version { "equal" } else { "less" };
                        return Err(Error::new(
                            format!("version is {m} than previous op"),
                            op.op_name,
                        ));
                    }
                }
                None => op.since = Some(LitInt::new(version)),
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
