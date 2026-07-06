use crate::prelude::*;
use crate::interface::*;

pub struct Constructor<'a> {
    pub iface: &'a Interface,
    pub rq: &'a Ops,
    pub ev: &'a Ops,
}

impl<'a> Constructor<'a> {
    pub fn new(iface: &'a Interface, rq: &'a Ops, ev: &'a Ops) -> Self {
        Self { iface, rq, ev }
    }
}

fn gen_op(op: &Op) -> impl Iterator<Item = TokenTree> {
    let Op { wl_name, op_name, lf_ph, op_span, .. } = op;

    let wl_name = if KEYWORDS.contains(&wl_name.as_str()) {
        Ident::new_raw(wl_name.as_str(), *op_span)
    } else {
        wl_name.clone().spanned(*op_span)
    };

    let lf = lf_ph.named();

    let doc = (op.since.is_some() || op.is_destructor).then_stream(|| {
        let doc = match (op.since.as_ref(), op.is_destructor) {
            (None, false) => unreachable!(),
            (None, true) => Literal::string(" destructor"),
            (Some(s), false) => Literal::string(&format!(" since={}", s.get())),
            (Some(s), true) => Literal::string(&format!(" since={}, destructor", s.get())),
        };
        g!(#[doc = #doc])
    });

    let args = op.args.iter().flat_map(|Arg { name, span, opt, ty, .. }|{
        let name = name.clone().spanned(*span);
        let ty = ty.generate();
        let ty = if *opt {
            Either::Left(g!(Option<#ty>))
        } else {
            Either::Right(Some(ty).into_iter())
        };
        g!(,#name: @ty)
    });
    let ctor = op.args.iter().flat_map(|Arg { name, .. }|{
        let mut name = name.clone();
        name.set_span(Span::call_site());
        g!(#name,)
    });

    g! {
        @doc
        pub fn #wl_name @lf(&self @args) -> Message<#op_name @lf> {
            Message::new(self.object_id(), #op_name { @ctor })
        }
    }
}

impl Constructor<'_> {
    pub fn gen_constructor(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.iface.iface_name;
        let rq = self.rq.iter().flat_map(gen_op);
        let ev = self.ev.iter().flat_map(gen_op);
        g! {
            impl<M> Object<#name, M> {
                @rq
                @ev
            }
        }
    }
}
