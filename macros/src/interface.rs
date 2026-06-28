use crate::prelude::*;

use interface::Interface;
use op::{Arg, Op, OpKind};
use ops::Ops;
use opcode::OpCode;
use constructor::Constructor;
use message::Message;
use enums::{Enums, Enum};

pub mod derive;

mod interface;
mod op;
mod ops;
mod opcode;
mod constructor;
mod message;
mod enums;

pub fn impl_interface(mut parser: Parser) -> Result<TokenStream, Error> {
    let iface = parser.parse::<Interface>()?;
    let rq = Ops::parse(OpKind::Request, &mut parser)?;
    let ev = Ops::parse(OpKind::Event, &mut parser)?;
    let en = Enums::parse(&mut parser)?;
    let rqop = OpCode::new(&rq);
    let evop = OpCode::new(&ev);
    let ctr = Constructor::new(&iface, &rq, &ev);

    let module = {
        let Interface { iface, wl_iface, .. } = &iface;
        g! {
            pub use #wl_iface::#iface;
            pub mod #wl_iface
        }
    };

    parser.check_empty()?;

    Ok(g!(use super::*;)
        .chain(iface.gen_struct())
        .chain(iface.gen_impl_marker())
        .chain(iface.gen_wl_global())
        .chain(rqop.gen_enum())
        .chain(rqop.gen_display())
        .chain(rqop.gen_opcode_trait())
        .chain(evop.gen_enum())
        .chain(evop.gen_display())
        .chain(evop.gen_opcode_trait())
        .chain(gen_messages(&iface.iface, &rqop))
        .chain(gen_messages(&iface.iface, &evop))
        .chain(ctr.gen_constructor())
        .chain(en.enums.iter().flat_map(gen_enum))
        .map_group(Delimiter::Brace)
        .chain_back(module)
        .collect())
}

fn gen_enum(en: &Enum) -> impl Iterator<Item = TokenTree> {
    en.gen_enum()
        .chain(en.gen_wl_enum())
        .chain(en.gen_display())
        .chain(en.gen_consts())
        .chain(en.gen_impl_flags())
        .chain(en.gen_bit_ops())
}

fn gen_messages(iface: &Ident, opcode: &OpCode) -> impl Iterator<Item = TokenTree> {
    opcode
        .ops
        .iter()
        .map(move |op| Message::new(opcode, op))
        .flat_map(move |m| {
            m.gen_struct()
                .chain(m.gen_as_interface(iface))
                .chain(m.gen_as_opcode())
                .chain(m.gen_as_newid())
                .chain(m.gen_wl_message())
                .chain(m.gen_decode())
                .chain(m.gen_encode_payload())
                .chain(m.gen_display())
        })
}

fn attr(parser: &mut Parser) -> Result<Option<Parser>, Error> {
    match parser.next_punct_of('#') {
        Some(_) => Ok(Some(parser.group_of(Delimiter::Bracket)?.body_parser())),
        None => Ok(None),
    }
}
