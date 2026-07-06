use crate::prelude::*;
use crate::interface::*;

pub struct Message<'a> {
    pub is_request: bool,
    pub iface: &'a Interface,
    pub op: &'a Op,
    pub fd: Option<&'a Arg>,
}

fn encodables(op: &Op) -> std::iter::Filter<std::slice::Iter<'_, Arg>, fn(&&Arg) -> bool> {
    op.args.iter().filter(|a|!a.ty.is_fd())
}

impl<'a> Message<'a> {
    pub fn new(iface: &'a Interface, op: &'a Op) -> Self {
        let is_request = matches!(op.kind, OpKind::Request);
        let fd = op.fd_idx.map(|i|&op.args[i]);
        Self {
            is_request,
            iface,
            op,
            fd,
        }
    }

    pub fn gen_struct(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.op_name;
        let fields = self.op.args.iter().flat_map(|Arg { name, opt, ty, .. }| {
            let ty = ty.generate();
            let ty = if *opt {
                Either::Left(g!(Option<#ty>))
            } else {
                Either::Right(Some(ty).into_iter())
            };
            g!(pub #name: @ty,)
        });
        let lf = self.op.lf_ph.named();
        g! {
            #[derive(Debug, Clone)]
            pub struct #name @lf {
                @fields
            }
        }
    }

    pub fn gen_as_interface(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let iface_name = &self.iface.iface_name;
        let op_name = &self.op.op_name;
        let lf_ph = self.op.lf_ph;
        g! {
            impl AsInterface for #op_name @lf_ph {
                #[inline]
                fn interface(&self) -> Interface {
                    Interface::#iface_name
                }
            }
        }
    }

    pub fn gen_as_newid(&self) -> impl Iterator<Item = TokenTree> + use<> {
        self.op
            .args
            .iter()
            .find_map(|a| a.ty.as_new_id().map(|i| (&a.name, i)))
            .map_stream(|(field, new_iface)| {
                let name = &self.op.op_name;
                g! {
                    impl AsNewId for #name {
                        type Interface = #new_iface;

                        #[inline]
                        fn new_id(&self) -> NewId<Self::Interface> {
                            self.#field
                        }
                    }
                }
            })
    }

    pub fn gen_as_opcode(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.op_name;
        let lf_ph = self.op.lf_ph;
        let opkind = Ident::new(match self.op.kind {
            OpKind::Request => "RequestOp",
            OpKind::Event => "EventOp",
        }, Span::call_site());

        g! {
            impl AsOpCode for #name @lf_ph {
                type OpCode = #opkind;
                const OPCODE: Self::OpCode = #opkind::#name;
                const OPNAME: &'static str = Self::OPCODE.name();
            }
        }
    }

    pub fn gen_wl_message(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let iface_name = &self.iface.iface_name;
        let name = &self.op.op_name;
        let is_request = Bool(self.is_request);
        let lf_ph = self.op.lf_ph;
        let destructor = self.op.is_destructor.then_stream(|| g! {
            const IS_DESTRUCTOR: bool = #TRUE;
        });
        let since = self.op.since.as_ref().map_stream(|since| {
            g!(const SINCE: Version = Version::new(#since).unwrap();)
        });
        g! {
            impl WlMessage for #name @lf_ph {
                type WlInterface = #iface_name;
                const IS_REQUEST: bool = #is_request;
                @destructor
                @since
            }
        }
    }

    pub fn gen_decode_payload(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.op_name;
        let lf_ph = self.op.lf_ph;
        let lf = self.op.lf_ph.named();

        let ret = self.op.args.iter().flat_map(|Arg { name, ty, .. }|{
            let read = (!ty.is_fd()).then_stream(||g!(: reader.read()?));
            g!(#name @read,)
        });

        let fd_len = Literal::usize_unsuffixed(self.fd.is_some() as usize);
        let fd_arg = self.fd.map(|arg|arg.name.clone());
        let reader_mut = (!self.op.args.is_empty()).then(||Ident::new("mut", Span::call_site()));

        g! {
            impl DecodePayload for #name @lf_ph {
                type Output<'a> = #name @lf;

                type Fd = [i32; #fd_len];

                #[inline]
                fn decode_payload<'a>(?reader_mut reader: Reader<'a>, [?fd_arg]: Self::Fd) -> Result<Self::Output<'a>, DecodeError> {
                    Ok(#name { @ret })
                }
            }
        }
    }

    pub fn gen_encode_payload(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.op_name;
        let lf_ph = self.op.lf_ph;

        let sum = encodables(self.op).flat_map(|Arg { name, .. }|{
            g!(+ self.#name.size())
        });
        let write = encodables(self.op).flat_map(|Arg { name, .. }|{
            g!(.write(self.#name))
        });
        let fd = self.fd.map_stream(|Arg { name, .. }|g! {
            #[inline]
            fn fd(&self) -> Option<i32> {
                Some(self.#name)
            }
        });

        g! {
            impl EncodePayload for #name @lf_ph {
                #[inline]
                fn size(&self) -> u16 {
                    #ZERO @sum
                }
                #[inline]
                fn encode_payload(self, writer: Writer) {
                    writer @write;
                }
                @fd
            }
        }
    }

    pub fn gen_display(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.op_name;
        let lf_ph = self.op.lf_ph;

        let fields = self.op.args.iter().flat_map(|Arg { name, wl_string, .. }|{
            g!(.field(#wl_string, &self.#name))
        });

        g! {
            impl display::AsDisplay for #name @lf_ph {
                #[inline]
                fn display(&self) -> impl std::fmt::Display {
                    std::fmt::from_fn(|f|{
                        f.debug_msg(
                            <Self as WlMessage>::WlInterface::INTERFACE_NAME,
                            Self::OPNAME,
                        )
                            @fields
                            .finish()
                    })
                }
            }
        }
    }
}
