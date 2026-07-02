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

mod opcode;
mod constructor;
mod message;
mod enums;

pub fn impl_interface(mut parser: Parser) -> Result<TokenStream, Error> {
    let iface = parser.parse::<Interface>()?;
    let rq = Ops::parse(OpKind::Request, &mut parser)?;
    let ev = Ops::parse(OpKind::Event, &mut parser)?;
    let enums = Enums::parse(&mut parser)?;

    let rqop = OpCode::new(&rq);
    let evop = OpCode::new(&ev);
    let ctr = Constructor::new(&iface, &rq, &ev);

    let Interface { iface_name, wl_iface, .. } = &iface;
    let premodule = g! {
        pub use #wl_iface::#iface_name;
        pub mod #wl_iface
    };
    let prelude = g! {
        use super::*;
        pub type InterfaceType = #iface_name;
    };

    parser.check_empty()?;

    Ok(prelude
        .chain(iface.generate())
        .chain(rqop.gen_enum())
        .chain(rqop.gen_display())
        .chain(rqop.gen_opcode_trait())
        .chain(evop.gen_enum())
        .chain(evop.gen_display())
        .chain(evop.gen_opcode_trait())
        .chain(gen_messages(&iface, &rqop))
        .chain(gen_messages(&iface, &evop))
        .chain(ctr.gen_constructor())
        .chain(enums.generate())
        .map_group(Delimiter::Brace)
        .chain_back(premodule)
        .collect())
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
