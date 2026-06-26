use crate::prelude::*;
use crate::interface::*;

pub struct ExtTrait<'a> {
    pub iface: &'a Interface,
    pub rq: &'a Ops,
    pub ev: &'a Ops,
}

impl<'a> ExtTrait<'a> {
    pub fn new(iface: &'a Interface, rq: &'a Ops, ev: &'a Ops) -> Self {
        Self { iface, rq, ev }
    }
}

fn gen_ops(ops: &Ops) -> impl Iterator<Item = TokenTree> + Clone {
    ops.iter().flat_map(|op|{
        let Op { wl_name, name, lf_ph, .. } = op;
        let lf = lf_ph.as_ref().map_stream(|_|g!(<'a>));
        let args = op.args.iter().flat_map(|Arg { name, ty, .. }|{
            let mut name = name.clone();
            name.set_span(Span::call_site());
            let ty = ty.generate();
            g!(,#name: #ty)
        });
        let ctor = op.args.iter().flat_map(|Arg { name, .. }|{
            let mut name = name.clone();
            name.set_span(Span::call_site());
            g!(#name,)
        });
        g! {
            fn #wl_name @lf(&self @args) -> Message<#name @lf> {
                Message::new(self, #name { @ctor })
            }
        }
    })
}

impl ExtTrait<'_> {
    pub fn gen_ext_trait(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.iface.iface;
        let rq = gen_ops(self.rq);
        let ev = gen_ops(self.ev);
        g! {
            pub trait Ext: Sized + AsObjectId {
                @rq
                @ev
            }

            impl<O: AsObject<#name>> Ext for O { }
        }
    }
}
