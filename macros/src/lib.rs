use proc_macro::*;

use crate::codegen::{ToTokens, generate};
use crate::error::Error;
use crate::parser::Parser;
use crate::syntax::{Attribute, Attributes, Lifetimes};

mod parser;
mod syntax;
mod codegen;
mod error;

/// Implement `FromObjectId`, `AsObjectId` and `AsInterface`.
#[proc_macro_derive(Interface)]
pub fn interface(tokens: TokenStream) -> TokenStream {
    impl_interface(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

/// Implement `OpCode`.
#[proc_macro_derive(OpCode)]
pub fn op_code(tokens: TokenStream) -> TokenStream {
    impl_op_code(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

/// Implement `Decode`, `Encode`, `AsInterface`, and add constructor of the message in the interface
/// object.
#[proc_macro_derive(Message, attributes(request, event, fd))]
pub fn message(tokens: TokenStream) -> TokenStream {
    impl_message(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

/// Define an enum containing all implemented interfaces.
///
/// Define module that reexports all interface modules to upper camel case.
///
/// For interface that want to be added to the enum, but not yet implemented, add the `#[todo]`
/// attribute.
///
/// The enum will also have a method that returns the snake cased wayland name.
///
/// ```ignore
/// protocol! {
///     /// Reexport interfaces as upper camel case.
///     pub mod interfaces;
///
///     #[derive(Debug, Clone, Copy, PartialEq, Eq)]
///     pub enum Interface;
///
///     pub mod wl_display;
///     pub mod wl_registry;
///
///     // variant will be defined, but module will not be declared
///     #[todo]
///     pub mod wp_presentation;
/// }
///
/// let wl_display = Interface::WlDisplay;
/// assert_eq!(wl_display.name(), "wl_display");
/// ```
#[proc_macro]
pub fn protocol(tokens: TokenStream) -> TokenStream {
    impl_protocol(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

// ===== implementations =====

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
                Some(Self::#last_variant)
            } else {
                None
            }
        },
        _ => generate! {
            if op as u8 <= Self::#last_variant as u8 {
                Some(unsafe { std::mem::transmute::<u8, Self>(op as u8) })
            } else {
                None
            }
        },
    };

    Ok(generate! {
        impl OpCode for #name {
            #[inline]
            fn from_op(op: u16) -> Option<Self> {
                #from_op
            }

            #[inline]
            fn to_op(self) -> u16 {
                self as u16
            }
        }
    })
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
    let mut ctor_args = TokenStream::new();
    let mut ctor = TokenStream::new();
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
        let mut ty = TokenStream::new();
        while let Some(tree) = body.next() {
            if let TokenTree::Punct(punct) = &tree
                && punct.as_char() == ','
            {
                break;
            }
            tree.into_tokens(&mut ty)
        }

        if len == 1 {
            dec_1 = Some(name.clone());
            enc_1 = Some(name.clone());
        }

        name.to_tokens(&mut dec_read);

        if is_fd {
            generate!(let #&name = decoder.pop_fd()?;).into_tokens(&mut dec_fd);
            generate!(,).into_tokens(&mut dec_read);
            generate!(encoder.push_fd(self.#&name);).into_tokens(&mut enc_fd);
        } else {
            col.into_tokens(&mut dec_read);
            generate!(reader.read()?,).into_tokens(&mut dec_read);

            generate!(+ self.#&name.size()).into_tokens(&mut enc_len);
            generate!(.write(self.#&name)).into_tokens(&mut enc_write);
        }

        generate!(, #&name: #ty).into_tokens(&mut ctor_args);
        generate!(#name,).into_tokens(&mut ctor);
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

    let ctor = if ctor.is_empty() {
        None
    } else {
        let mname = Ident::new(&to_snake(&name.to_string()), name.span());
        Some(generate! {
            impl #&iface {
                pub fn #mname #&lf (&self #ctor_args) -> Message<#&name #&lf> {
                    Message::new(self, #&name { #ctor })
                }
            }
        })
    };

    Ok(generate! {
        #ctor

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

fn impl_protocol(mut parser: Parser) -> Result<TokenStream, Error> {
    let mod_attrs = parser.parse::<Attributes>()?;
    let mod_vis = parser.ident_of("pub")?;
    let mod_kw = parser.ident_of("mod")?;
    let mod_name = parser.ident()?;
    let _ = parser.punct_of(';')?;

    let attrs = parser.parse::<Attributes>()?;
    let vis = parser.ident_of("pub")?;
    let enum_kw = parser.ident_of("enum")?;
    let name = parser.ident()?;
    let _ = parser.punct_of(';')?;

    let mut mod_declare = TokenStream::new();
    let mut reexports = TokenStream::new();
    let mut variants = TokenStream::new();
    let mut names = TokenStream::new();
    let mut len = 0;

    loop {
        let (vis, attr) = if parser.is_punct_of('#').is_some() {
            let attr = parser.parse::<Attribute>()?;
            let vis = parser.ident_of("pub")?;
            (vis, Some(attr))
        } else if let Some(vis) = parser.next_ident_of("pub") {
            (vis, None)
        } else {
            break;
        };
        len += 1;

        let is_todo = attr
            .and_then(|attr| Parser::new(attr.tokens).next_ident_of("todo"))
            .is_some();
        let mod_kw = parser.ident_of("mod")?;
        let name = parser.ident()?;
        let semi = parser.punct_of(';')?;

        let name_string = name.to_string();
        let name_camel = Ident::new(&to_camel(&name_string), Span::call_site());

        variants.extend([
            TokenTree::from(name_camel.clone()),
            Punct::new(',', Spacing::Alone).into(),
        ]);
        names.extend([
            TokenTree::from(Literal::string(&name_string)),
            Punct::new(',', Spacing::Alone).into(),
        ]);

        if !is_todo {
            generate!(pub use super::#&name as #name_camel;).into_tokens(&mut reexports);
            mod_declare.extend([
                TokenTree::from(vis),
                mod_kw.into(),
                name.into(),
                semi.into(),
            ])
        }
    }

    let len = Literal::usize_unsuffixed(len);

    Ok(generate! {
        #attrs
        #vis #enum_kw #&name {
            #variants
        }

        impl #&name {
            #[doc = " Returns lower cased name of current interface."]
            #[inline]
            pub fn name(&self) -> &'static str {
                static LOOKUP: [&'static str; #len] = [#names];
                unsafe { LOOKUP.get_unchecked(*self as usize) }
            }
        }

        impl std::fmt::Display for #name {
            #[inline]
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                self.name().fmt(f)
            }
        }

        #mod_declare

        #mod_attrs
        #mod_vis #mod_kw #mod_name {
            #reexports
        }
    })
}

fn to_camel(string: &str) -> String {
    let mut output = String::with_capacity(string.len());
    let mut chars = string.chars();
    if let Some(first) = chars.next() {
        output.extend(first.to_uppercase());
    }
    while let Some(ch) = chars.next() {
        if ch == '_' {
            if let Some(next) = chars.next() {
                output.extend(next.to_uppercase());
            }
        } else {
            output.push(ch);
        }
    }
    output
}

fn to_snake(string: &str) -> String {
    let mut output = String::with_capacity(string.len());
    let mut chars = string.chars();
    if let Some(first) = chars.next() {
        output.extend(first.to_lowercase());
    }
    for ch in chars {
        if ch.is_uppercase() {
            output.extend(std::iter::once('_').chain(ch.to_lowercase()));
        } else {
            output.push(ch);
        }
    }
    output
}
