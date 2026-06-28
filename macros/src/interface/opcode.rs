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
        let name = &self.name;
        let from = match &self.ops[..] {
            [] => Either::Left(Either::Left(g!(None))),
            [Op { name, .. }] => Either::Left(Either::Right(g! {
                if op == #ZERO { Some(Self::#name) } else { None }
            })),
            [.., Op { name, .. }] => {
                Either::Right(g! {
                    if op as u8 <= Self::#name as u8 {
                        Some(unsafe { std::mem::transmute::<u8, Self>(op as u8) })
                    } else {
                        None
                    }
                })
            },
        };
        g! {
            impl OpCode for #name {
                #[inline]
                fn from_op(op: u16) -> Option<Self> {
                    @from
                }

                #[inline]
                fn to_op(self) -> u16 {
                    self as u16
                }
            }
        }
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
