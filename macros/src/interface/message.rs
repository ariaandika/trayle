use crate::prelude::*;
use crate::interface::*;

pub struct Message<'a> {
    has_lf: bool,
    has_fd: Option<&'a Arg>,
    is_request: bool,
    opkind: Ident,
    op: &'a Op,
}

fn encodables(op: &Op) -> std::iter::Filter<std::slice::Iter<'_, Arg>, fn(&&Arg) -> bool> {
    op.args.iter().filter(|a|!a.ty.is_fd())
}

impl<'a> Message<'a> {
    pub fn new(opcode: &OpCode, op: &'a Op) -> Self {
        let has_lf = op.args.iter().any(|e| e.ty.is_lf());
        let has_fd = op.args.iter().find(|e| e.ty.is_fd());
        let opkind = opcode.name.clone();
        let is_request = matches!(opcode.ops.kind, OpKind::Request);
        Self {
            has_lf,
            has_fd,
            is_request,
            opkind,
            op,
        }
    }

    pub fn gen_struct(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.name;
        let fields = encodables(self.op).flat_map(|Arg { name, /* opt, */ ty, .. }| {
            let ty = ty.generate();
            // FIX:
            // - blocker: change impl Read for Option<Object>
            // - blocker: remove the old wayland definition
            // let ty = if *opt {
            //     Either::Left(g!(Option<#ty>))
            // } else {
            //     Either::Right(Some(ty).into_iter())
            // };
            g!(pub #name: #ty,)
        });
        let lf = stream_if(self.has_lf, || g!(<'a>));
        g! {
            #[derive(Debug, Clone)]
            pub struct #name @lf {
                @fields
            }
        }
    }

    pub fn gen_as_interface(&self, iface: &Ident) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.name;
        let lf_ph = stream_if(self.has_lf, || g!(<'_>));
        g! {
            impl AsInterface for #name @lf_ph {
                #[inline]
                fn interface(&self) -> Interface {
                    Interface::#iface
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
                let name = &self.op.name;
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
        let name = &self.op.name;
        let opkind = &self.opkind;
        let lf_ph = stream_if(self.has_lf, || g!(<'_>));
        let wl_string = Literal::string(self.op.wl_name.as_str());
        g! {
            impl AsOpCode for #name @lf_ph {
                type OpCode = #opkind;
                const OPCODE: Self::OpCode = #opkind::#name;
                const OPNAME: &'static str = #wl_string;
            }
        }
    }

    pub fn gen_wl_message(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.name;
        let is_request = Bool(self.is_request);
        let lf_ph = stream_if(self.has_lf, || g!(<'_>));
        let destructor = stream_if(self.op.is_destructor, || g! {
            const IS_DESTRUCTOR: bool = #TRUE;
        });
        let since = self.op.since.as_ref().map_stream(|since| g! {
            const SINCE: Version = Version::new(#since).unwrap();
        });
        g! {
            impl WlMessage for #name @lf_ph {
                const IS_REQUEST: bool = #is_request;
                @destructor
                @since
            }
        }
    }

    pub fn gen_decode(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.name;
        let cmut = stream_if(self.has_fd.is_some(), ||g!(mut));
        let lf = stream_if(self.has_lf, || g!(<'a>));
        let lf_ph = stream_if(self.has_lf, || g!(<'_>));

        let fd = self.has_fd.as_ref().map_stream(|Arg { name, .. }|{
            g!(let #name = dec.pop_fd()?;)
        });
        let ret = self.op.args.iter().flat_map(|Arg { name, ty, .. }|{
            let read = stream_if(!ty.is_fd(), ||g!(: reader.read()?));
            g!(#name @read,)
        });

        g! {
            impl Decode for #name @lf_ph {
                type Output<'a> = #name @lf;

                #[inline]
                fn decode<'a>(@cmut dec: Decoder<'a>) -> Result<Self::Output<'a>, DecodeError> {
                    @fd
                    let mut reader = dec.reader();
                    Ok(#name { @ret })
                }
            }
        }
    }

    pub fn gen_encode_payload(&self) -> impl Iterator<Item = TokenTree> + use<> {
        let name = &self.op.name;
        let lf_ph = stream_if(self.has_lf, || g!(<'_>));

        let sum = encodables(self.op).flat_map(|Arg { name, .. }|{
            g!(+ self.#name.size())
        });
        let write = encodables(self.op).flat_map(|Arg { name, .. }|{
            g!(.write(self.#name))
        });
        let fd = self.has_fd.map_stream(|Arg { name, .. }|g! {
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
        let name = &self.op.name;
        let lf_ph = stream_if(self.has_lf, || g!(<'_>));

        g! {
            impl display::AsDisplay for #name @lf_ph {
                #[inline]
                fn display(&self) -> impl std::fmt::Display {
                    std::fmt::from_fn(|f|std::fmt::Debug::fmt(self, f))
                }
            }
        }
    }
}
