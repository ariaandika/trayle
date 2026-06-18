use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let attrs = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("struct")?;
    let name = parser.ident()?;
    let lf = parser.parse::<Lifetimes>()?;
    let body = parser
        .next_group_of(Delimiter::Brace)
        .map(|e| e.stream())
        .unwrap_or_default();

    let meta = Metadata::new(attrs)?;
    let lf_ph = if lf.lfs.is_empty() {
        token_stream!()
    } else {
        token_stream!(<'_>)
    };

    let mut encodable = 0;
    let mut len = 0;

    let mut body = Parser::new(body);
    let mut constructor = GenConstructor::default();
    let mut decode = GenDecode::default();
    let mut encode = GenEncode::default();
    let mut display = GenDisplay::default();

    while let Some(field) = body.separated(',')? {
        let FieldNamed { attrs, ident, ty, .. } = field;
        let is_fd = attrs
            .attrs
            .into_iter()
            .any(|e| e.ident.as_str() == "fd");
        constructor.add_field(&ident, ty);
        decode.add_field(is_fd, encodable, &ident);
        encode.add_field(is_fd, encodable, &ident);
        display.add_field(len, is_fd, &ident);
        encodable += (!is_fd) as usize;
        len += 1;
    }

    Ok(meta
        .generate(&name, &lf_ph)
        .chain(constructor.generate(len, &name, &lf, &meta.iface))
        .chain(decode.generate(len, encodable, &name, &lf, &lf_ph))
        .chain(encode.generate(&name, &lf_ph))
        .chain(display.generate(&name, &lf_ph))
        .collect())
}

// ===== impl =====

struct Metadata {
    opkind: Ident,
    iface: Ident,
}

impl Metadata {
    fn new(attrs: Attributes) -> Result<Self, Error> {
        let mut opkind = None;
        for Attribute { ident, meta, .. } in attrs.attrs {
            let mut body = meta.try_seq()?.body_parser();
            let Some(iface) = body.next_ident() else {
                continue;
            };
            if opkind.is_none() {
                let opkind_ = match ident.as_str() {
                    "request" => Ident::new("RequestOp", ident.span()),
                    "event" => Ident::new("EventOp", ident.span()),
                    _ => continue,
                };
                opkind = Some((opkind_, iface));
            }
        }
        let Some((opkind, iface)) = opkind else {
            return Err(Error::new(
                "`request` or `event` attribute with interface name is required",
            ));
        };
        Ok(Self {
            opkind,
            iface,
        })
    }

    fn generate(&self, name: &Ident, lf_ph: &TokenStream) -> impl Iterator<Item = TokenTree> {
        let Self { opkind, iface } = self;
        let wl_name = Literal::string(&to_snake(name.as_str()));
        generate! {
            impl AsInterface for #name @lf_ph {
                #[inline]
                fn interface(&self) -> Interface {
                    Interface::#iface
                }
            }

            impl AsOpCode for #name @lf_ph {
                type OpCode = #opkind;

                const OPCODE: Self::OpCode = #opkind::#name;

                const OPNAME: &'static str = #wl_name;
            }
        }
    }
}

#[derive(Default)]
struct GenConstructor {
    args: TokenStream,
    construct: TokenStream,
}

impl GenConstructor {
    fn add_field(&mut self, ident: &Ident, ty: TokenStream) {
        self.args.extend(generate!(, #ident: @ty));
        self.construct.extend(generate!(#ident,));
    }

    fn generate(
        self,
        len: usize,
        name: &Ident,
        lf: &Lifetimes,
        iface: &Ident,
    ) -> impl Iterator<Item = TokenTree> {
        if len > 6 {
            return None.into_iter().flatten();
        }
        let mname = to_snake(name.as_str());
        if super::KEYWORDS.contains(&mname.as_str()) {
            return None.into_iter().flatten();
        }
        let mname = Ident::new(&mname, Span::call_site());
        let Self { args, construct, .. } = self;
        Some(generate! {
            impl #iface {
                #[inline]
                pub fn #mname@lf(&self @args) -> Encodable<#name @lf> {
                    Encodable::new(self, #name { @construct })
                }
            }
        })
        .into_iter()
        .flatten()
    }
}

#[derive(Default)]
struct GenDecode {
    dec_1: Option<Ident>,
    read: TokenStream,
    fd: TokenStream,
}

impl GenDecode {
    fn add_field(&mut self, is_fd: bool, encodable: usize, ident: &Ident) {
        if encodable == 0 {
            self.dec_1 = Some(ident.clone());
        }

        self.read.push(ident.clone());

        if is_fd {
            self.fd.extend(generate!(let #ident = decoder.pop_fd()?;));
            self.read.extend(gentoken!(,));
        } else {
            self.read.extend(generate!(: reader.read()?,));
        }
    }

    fn generate(
        self,
        len: usize,
        encodable: usize,
        name: &Ident,
        lf: &Lifetimes,
        lf_ph: &TokenStream,
    ) -> impl Iterator<Item = TokenTree> {
        let Self { dec_1, read: field, fd } = self;
        let coding_mut = if fd.is_empty() { None } else { gentoken!(mut) };
        let reader = match (len, encodable) {
            (0, _) => generate!(let _ = decoder.reader();).collect::<TokenStream>(),
            (1, 1) => generate!().collect(),
            _ => generate!(let mut reader = decoder.reader();).collect(),
        };
        let ret = match (len, encodable) {
            (0, _) => generate!({}).collect::<TokenStream>(),
            (1, 1) => generate!({ ?dec_1: decoder.read()? }).collect(),
            _ => generate!({ @field }).collect(),
        };
        generate! {
            impl Decode for #name @lf_ph {
                type Output<'a> = #name @lf;

                #[inline]
                fn decode<'a>(?coding_mut decoder: Decoder<'a>) -> Result<Self::Output<'a>, DecodeError> {
                    @fd
                    @reader
                    Ok(#name @ret)
                }
            }
        }
    }
}

#[derive(Default)]
struct GenEncode {
    len: TokenStream,
    fd: TokenStream,
    write: TokenStream,
}

impl GenEncode {
    fn add_field(&mut self, is_fd: bool, encodable: usize, ident: &Ident) {
        if is_fd {
            self.fd.extend(generate!(self.#ident,));
        } else {
            let plus = if encodable == 0 {
                None
            } else {
                gentoken!(+)
            };
            self.len.extend(generate!(?plus self.#ident.size()));
            self.write.extend(generate!(.write(self.#ident)));
        }
    }

    fn generate(self, name: &Ident, lf_ph: &TokenStream) -> impl Iterator<Item = TokenTree> {
        let Self { len, fd, write } = self;

        let len = if len.is_empty() {
            token_stream!(#ZERO)
        } else {
            len
        };

        let fds = if fd.is_empty() {
            None
        } else {
            Some(generate! {
                #[inline]
                fn fds(&self) -> impl IntoIterator<Item = i32> {
                    [@fd]
                }
            })
        }
        .into_iter()
        .flatten();

        generate! {
            impl Encode for #name @lf_ph {
                #[inline]
                fn size(&self) -> u16 {
                    @len
                }

                #[inline]
                fn encode(self, writer: Writer) {
                    writer @write;
                }

                @fds
            }
        }
    }
}

#[derive(Default)]
struct GenDisplay {
    tokens: TokenStream,
}

impl GenDisplay {
    fn add_field(&mut self, i: usize, is_fd: bool, ident: &Ident) {
        if i != 0 {
            let comma = Literal::character(',');
            self.tokens
                .extend(generate!(std::fmt::Display::fmt(&#comma, f)?;));
        }
        if is_fd {
            self.tokens.extend(generate!(std::fmt::Display::fmt(&"<fd>", f)?;));
        } else {
            self.tokens.extend(generate!(crate::wayland::display::fmt_me(&self.#ident, f)?;));
        }
    }

    fn generate(self, name: &Ident, lf_ph: &TokenStream) -> impl Iterator<Item = TokenTree> {
        let Self { tokens } = self;
        generate! {
            impl display::AsDisplay for #name @lf_ph {
                #[inline]
                fn display(&self) -> impl std::fmt::Display {
                    std::fmt::from_fn(|f|{
                        @tokens
                        Ok(())
                    })
                }
            };
        }
    }
}
