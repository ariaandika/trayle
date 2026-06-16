use crate::prelude::*;

pub fn process(mut parser: Parser) -> Result<TokenStream, Error> {
    let attrs = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("struct")?;
    let name = parser.ident()?;
    let lf_token = Lifetimes::parse_opt(&mut parser)?;
    let plf = lf_token.as_ref().map(|_|generate!(<'_>).collect::<TokenStream>()).unwrap_or_default();
    let lf = lf_token.map(|e|e.into_iter().collect()).unwrap_or_else(TokenStream::new);

    let kind_attr = attrs.attrs_parser().find_map(|mut parser|{
        let name = parser.next_ident()?;
        let opkind = match name.to_string().as_str() {
            "request" => Ident::new("RequestOp", name.span()),
            "event" => Ident::new("EventOp", name.span()),
            _ => return None,
        };
        let content = parser.next_group_of(Delimiter::Parenthesis)?;
        let iface = content.body_parser().next_ident()?;
        Some((opkind, iface))
    });
    let Some((opkind, iface)) = kind_attr else {
        return Err(Error::new(
            "`request` or `event` attribute with interface name is required".into(),
            name.span()
        ));
    };
    let wl_name = Literal::string(&to_snake(&name.to_string()));

    let body = parser.next_group_of(Delimiter::Brace).map(|e|e.stream()).unwrap_or_default();
    let mut body = Parser::new(body);

    let mut dec_1 = None;
    let mut dec_fd = TokenStream::new();
    let mut dec_read = TokenStream::new();
    let mut enc_len = TokenStream::from_iter(Some(TokenTree::from(Literal::u16_suffixed(0))));
    let mut enc_fd = TokenStream::new();
    let mut enc_write = TokenStream::new();
    let mut ctor_args = TokenStream::new();
    let mut ctor = TokenStream::new();
    let mut display = TokenStream::new();
    let mut encodable = 0;
    let mut len = 0;

    loop {
        let attrs = body.parse::<Attributes>()?;
        let Some(_) = body.next_ident_of("pub") else {
            break;
        };

        let is_fd = attrs.attrs.into_iter().any(|e|{
            match e.tokens.into_iter().next() {
                Some(TokenTree::Ident(id)) => id.to_string().as_str() == "fd",
                _ => false,
            }
        });

        encodable += (!is_fd) as usize;
        len += 1;

        let name = body.ident()?;
        let col = body.punct_of(':')?;

        // only support type which does not have comma in the middle
        let mut ty = TokenStream::new();
        while let Some(tree) = body.next() {
            if let TokenTree::Punct(punct) = &tree
                && punct.as_char() == ','
            {
                break;
            }
            ty.extend([tree]);
        }

        if encodable == 1 {
            dec_1 = Some(name.clone());
        }

        dec_read.push(name.clone());

        if len != 1 {
            let comma = Literal::character(',');
            display.extend(generate!(std::fmt::Display::fmt(&#comma, f)?;));
        }

        if is_fd {
            dec_fd.extend(generate!(let #name = decoder.pop_fd()?;));
            dec_read.extend(gentoken!(,));
            enc_fd.extend(generate!(self.#name,));
            display.extend(generate!(std::fmt::Display::fmt(&"<fd>", f)?;));
        } else {
            dec_read.push(col);
            dec_read.extend(generate!(reader.read()?,));
            enc_len.extend(generate!(+ self.#name.size()));
            enc_write.extend(generate!(.write(self.#name)));
            display.extend(generate!(crate::wayland::display::fmt_me(&self.#name, f)?;));
        }

        ctor_args.extend(generate!(, #name: @ty));
        ctor.extend(generate!(#name,));
    }

    let coding_mut = if dec_fd.is_empty() {
        None
    } else {
        gentoken!(mut)
    };

    let reader = match (len, encodable) {
        (0, _) => generate!(let _ = decoder.reader();).collect::<TokenStream>(),
        (1, 1) => generate!().collect(),
        _ => generate!(let mut reader = decoder.reader();).collect(),
    };
    let ret = match (len, encodable) {
        (0, _) => generate!({}).collect::<TokenStream>(),
        (1, 1) => generate!({ ?dec_1: decoder.read()? }).collect(),
        _ => generate!({ @dec_read }).collect(),
    };

    let enc_fds_impl = if enc_fd.is_empty() {
        TokenStream::new()
    } else {
        generate! {
            #[inline]
            fn fds(&self) -> impl IntoIterator<Item = i32> {
                [@enc_fd]
            }
        }.collect::<TokenStream>()
    };

    let ctor = if len <= 6
        && let mname = to_snake(&name.to_string())
        && !super::KEYWORDS.contains(&mname.as_str())
    {
        let mname = Ident::new(&mname, name.span());
        generate! {
            impl #iface {
                #[inline]
                pub fn #mname @lf (&self @ctor_args) -> Encodable<#name @lf> {
                    Encodable::new(self, #name { @ctor })
                }
            }
        }.collect::<TokenStream>()
    } else {
        TokenStream::new()
    };

    let gen1 = generate! {
        @ctor
 
        impl AsInterface for #name @plf {
            #[inline]
            fn interface(&self) -> Interface {
                Interface::#iface
            }
        }

        impl AsOpCode for #name @plf {
            type OpCode = #opkind;

            const OPCODE: Self::OpCode = #opkind::#name;

            const OPNAME: &'static str = #wl_name;
        }
    };

    let gen2 = generate! {
        impl Decode for #name @plf {
            type Output<'a> = #name @lf;

            #[inline]
            fn decode<'a>(?coding_mut decoder: Decoder<'a>) -> Result<Self::Output<'a>, DecodeError> {
                @dec_fd
                @reader
                Ok(#name @ret)
            }
        }

        impl Encode for #name @plf {
            #[inline]
            fn size(&self) -> u16 {
                @enc_len
            }

            #[inline]
            fn encode(self, writer: Writer) {
                writer @enc_write;
            }

            @enc_fds_impl
        }
    };

    let gen3 = generate! {
        impl display::AsDisplay for #name @plf {
            #[inline]
            fn display(&self) -> impl std::fmt::Display {
                std::fmt::from_fn(|f|{
                    @display
                    Ok(())
                })
            }
        };
    };

    Ok(gen1.chain(gen2).chain(gen3).collect())
}
