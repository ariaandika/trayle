use crate::prelude::*;

use interface::Interface;
use arg::Arg;
use op::{Op, OpKind};

use ops::Ops;
use opcode::OpCode;
use constructor::Constructor;
use message::Message;
use enums::Enums;


mod interface;
mod arg;
mod op;
mod ops;
mod enums;

mod opcode;
mod constructor;
mod message;

pub fn impl_interface(mut parser: Parser) -> Result<TokenStream, Error> {
    let iface = parser.parse::<Interface>()?;
    let rq = Ops::parse(OpKind::Request, &mut parser)?;
    let ev = Ops::parse(OpKind::Event, &mut parser)?;
    let enums = Enums::parse(&mut parser)?;

    parser.check_empty()?;

    let Interface { iface_name, .. } = &iface;
    let rqop = OpCode::new(&rq);
    let evop = OpCode::new(&ev);
    let ctr = Constructor::new(&iface, &rq, &ev);

    let main = iface
        .generate()
        .chain(rqop.gen_enum())
        .chain(rqop.gen_display())
        .chain(rqop.gen_opcode_trait())
        .chain(evop.gen_enum())
        .chain(evop.gen_display())
        .chain(evop.gen_opcode_trait())
        .chain(gen_messages(&iface, &rqop))
        .chain(gen_messages(&iface, &evop))
        .chain(ctr.gen_constructor())
        .chain(enums.generate());

    match &iface.mod_name {
        Some(mod_name) => Ok(g! {
            pub use #mod_name::#iface_name;
            pub mod #mod_name {
                use super::*;
                @main
            }
        }.collect()),
        None => Ok(main.collect()),
    }
}

fn gen_messages<'a>(
    iface: &'a Interface,
    opcode: &'a OpCode,
) -> impl Iterator<Item = TokenTree> + 'a {
    opcode
        .ops
        .iter()
        .map(move |op| Message::new(iface, op))
        .flat_map(move |m| {
            m.gen_struct()
                .chain(m.gen_as_interface())
                .chain(m.gen_as_opcode())
                .chain(m.gen_as_newid())
                .chain(m.gen_wl_message())
                .chain(m.gen_decode_payload())
                .chain(m.gen_encode_payload())
                .chain(m.gen_display())
        })
}
