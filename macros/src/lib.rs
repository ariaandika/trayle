use proc_macro::*;

use crate::codegen::{ToTokens, generate};
use crate::error::Error;
use crate::parser::Parser;
use crate::syntax::{Attributes, Lifetimes};

mod parser;
mod syntax;
mod codegen;
mod error;

/// Implement `FromObjectId`, `AsObjectId` and `AsInterface`.
#[proc_macro_derive(Interface)]
pub fn interface(tokens: TokenStream) -> TokenStream {
    impl_interface(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

fn impl_interface(mut parser: Parser) -> Result<TokenStream, Error> {
    let _attrs = parser.parse::<Attributes>()?;
    let _vis = parser.ident_of("pub")?;
    let _struct_kw = parser.ident_of("struct")?;
    let name = parser.ident()?;

    Ok(codegen::generate! {
        impl FromObjectId for #&name {
            #[inline]
            fn from_object_id(id: ObjectId) -> Self {
                Self { id }
            }
        }
        impl AsObjectId for #&name {
            #[inline]
            fn object_id(&self) -> ObjectId {
                self.id
            }
        }
        impl AsInterface for #&name {
            const INTERFACE: Interface = Interface::#name;
        }
    })
}

/// Implement `OpCode`.
#[proc_macro_derive(OpCode)]
pub fn op_code(tokens: TokenStream) -> TokenStream {
    impl_op_code(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

fn impl_op_code(mut parser: Parser) -> Result<TokenStream, Error> {
    let _ = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let mut body = Parser::new(parser.group_of(Delimiter::Brace)?.stream());

    let zero = Literal::u8_unsuffixed(0);

    let mut i = 1;
    let mut last_variant = body.next_ident().expect("empty enum");
    loop {
        body.next_punct_of(',');
        let Some(next_ident) = body.next_ident() else {
            break;
        };
        last_variant = next_ident;
        i += 1;
    }
    let from_op = match i {
        1 => generate! {
            if op == #zero {
                Ok(Self::#last_variant)
            } else {
                Err(WlError::UnknownOp)
            }
        },
        _ => generate! {
            if op as u8 <= Self::#last_variant as u8 {
                Ok(unsafe { std::mem::transmute::<u8, Self>(op as u8) })
            } else {
                Err(WlError::UnknownOp)
            }
        },
    };

    Ok(generate! {
        impl OpCode for #name {
            #[inline]
            fn from_op(op: u16) -> Result<Self, WlError> {
                #from_op
            }

            #[inline]
            fn to_op(self) -> u16 {
                self as u16
            }
        }
    })
}

/// Implement `Decode`, `Encode`, `AsInterface`.
#[proc_macro_derive(Message, attributes(request, event, fd))]
pub fn message(tokens: TokenStream) -> TokenStream {
    impl_message(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

fn impl_message(mut parser: Parser) -> Result<TokenStream, Error> {
    let attrs = parser.parse::<Attributes>()?;
    let _ = parser.ident_of("pub")?;
    let _ = parser.ident_of("struct")?;
    let name = parser.ident()?;
    let lf = Lifetimes::parse_opt(&mut parser)?;
    let plf = lf.as_ref().map(|_|generate!(<'_>));

    let kind_attr = attrs.attrs.into_iter().find_map(|e|{
        let mut parser = Parser::new(e.tokens);
        let name = parser.next_ident()?;
        let opkind = match name.to_string().as_str() {
            "request" => Ident::new("RequestOp", name.span()),
            "event" => Ident::new("EventOp", name.span()),
            _ => return None,
        };
        let content = parser.next_group_of(Delimiter::Parenthesis)?;
        let iface = Parser::new(content.stream()).next_ident()?;

        Some((opkind, iface))
    });
    let Some((opkind, iface)) = kind_attr else {
        return Err(Error::new(
            "`request` or `event` attribute with interface name is required".into(),
            name.span()
        ));
    };

    let mut body = Parser::new(parser.group_of(Delimiter::Brace)?.stream());
    let mut dec_1 = None;
    let mut dec_fd = TokenStream::new();
    let mut dec_read = TokenStream::new();
    let mut enc_1 = None;
    let mut enc_len = Literal::u16_suffixed(8).into_token_stream();
    let mut enc_fd = TokenStream::new();
    let mut enc_write = TokenStream::new();
    let mut len = 0;

    loop {
        let is_fd = if body.is_punct_of('#').is_some() {
            let attrs = body.parse::<Attributes>()?;
            let _ = body.next_ident_of("pub");

            attrs.attrs.into_iter().any(|e|{
                match e.tokens.into_iter().next() {
                    Some(TokenTree::Ident(id)) => id.to_string().as_str() == "fd",
                    _ => false,
                }
            })
        } else if body.next_ident_of("pub").is_some() {
            false
        } else {
            break;
        };
        len += (!is_fd) as usize;

        let name = body.ident()?;
        let col = body.punct_of(':')?;

        // only support type which does not have comma in the middle
        while let Some(tree) = body.next() {
            if let TokenTree::Punct(punct) = tree
                && punct.as_char() == ','
            {
                break;
            }
        }

        if len == 1 {
            dec_1 = Some(name.clone());
            enc_1 = Some(name.clone());
        }

        name.to_tokens(&mut dec_read);

        if is_fd {
            generate!(let #&name = decoder.pop_fd()?;).into_tokens(&mut dec_fd);
            generate!(,).into_tokens(&mut dec_read);
            generate!(encoder.push_fd(self.#name);).into_tokens(&mut enc_fd);
        } else {
            col.into_tokens(&mut dec_read);
            generate!(reader.read()?,).into_tokens(&mut dec_read);

            generate!(+ self.#&name.size()).into_tokens(&mut enc_len);
            generate!(.write(self.#name)).into_tokens(&mut enc_write);
        }
    }

    let coding_mut = if dec_fd.is_empty() {
        None
    } else {
        Some(generate!(mut))
    };

    let dec_impl = if len == 1 {
        generate! { Ok(#&name { #dec_1: decoder.read()? }) }
    } else {
        generate! {
            #dec_fd
            let mut reader = decoder.reader();
            Ok(#&name { #dec_read })
        }
    };

    let enc_impl = if len == 1 {
        generate! { encoder.encode1(self.#enc_1); }
    } else {
        generate! {
            use super::encode::Write;
            #enc_fd
            let len = #enc_len;
            unsafe { encoder.encode(len) } #enc_write;
        }
    };

    Ok(generate! {
        impl Decode for #&name #&plf {
            type Output<'a> = #&name #lf;

            #[inline]
            fn decode<'a>(#&coding_mut decoder: Decoder<'a>) -> Result<Self::Output<'a>, WlError> {
                #dec_impl
            }
        }

        impl Encode for Message<#&name #&plf> {
            const OPCODE: u16 = #opkind::#&name as u16;

            #[inline]
            fn encode(self, #coding_mut encoder: Encoder) {
                #enc_impl
            }
        }

        impl AsInterface for #name #plf {
            const INTERFACE: Interface = Interface::#iface;
        }
    })
}
