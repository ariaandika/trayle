use crate::prelude::*;

use crate::interface::{OpKind, Op, Ops};

pub struct OpCode<'a> {
    pub name: Ident,
    pub ops: &'a Ops,
}

impl<'a> OpCode<'a> {
    pub fn new(ops: &'a Ops) -> Self {
        let name = match ops.kind {
            OpKind::Request => "RequestOp",
            OpKind::Event => "EventOp",
        };
        let name = Ident::new(name, Span::call_site());
        Self { name, ops }
    }

    pub fn gen_enum(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.name;
        let variants = self.ops.iter().flat_map(|Op { name, .. }|{
            g!(#name,)
        });
        g! {
            #[derive(Debug, Clone, Copy)]
            pub enum #name {
                @variants
            }
        }
    }

    pub fn gen_opcode_trait(&self) -> impl Iterator<Item = TokenTree> {
        if self.ops.is_empty() {
            return None.into_iter().flatten();
        }
        let name = &self.name;
        let cmp = match &self.ops[..] {
            [] => unreachable!(),
            [_] => Either::Left(g!(op == #ZERO)),
            [.., Op { name, .. }] => Either::Right(g!(op as u8 <= Self::#name as u8)),
        };
        let cvt = match &self.ops[..] {
            [] => unreachable!(),
            [Op { name, .. }] => Either::Left(g!(Self::#name)),
            [.., _] => {
                Either::Right(g!(unsafe { std::mem::transmute::<u8, Self>(op as u8) }))
            },
        };
        Some(g! {
            impl OpCode for #name {
                #[inline]
                fn from_op(op: u16) -> Option<Self> {
                    if @cmp { Some(@cvt) } else { None }
                }

                #[inline]
                fn to_op(self) -> u16 {
                    self as u16
                }
            }
        }).into_iter().flatten()
    }

    pub fn gen_display(&self) -> impl Iterator<Item = TokenTree> {
        let name = &self.name;
        let names = self.ops.iter().flat_map(|Op { wl_name, name, .. }|{
            let wl_name = Literal::string(wl_name.as_str());
            g!(Self::#name => #wl_name,)
        });
        g! {
            impl #name {
                /// Returns the wayland name.
                #[inline]
                pub const fn name(self) -> &'static str {
                    match self {
                        @names
                    }
                }
            }
            impl std::fmt::Display for #name {
                #[inline]
                fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                    self.name().fmt(f)
                }
            }
        }
    }
}
