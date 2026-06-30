#![allow(clippy::module_inception)]
use proc_macro::Span;
use tree::{TokenStream, p};
use error::Error;
use parser::Parser;

mod span;
mod error;
mod tree;
mod tree_ext;
mod codegen;
mod to_tokens;
mod parser;

// ===== syntax =====

mod syntax;

// ===== implementations =====

mod prelude {
    pub(crate) use super::{Bool, KEYWORDS, to_camel, to_snake};
    pub use crate::span::*;
    pub use crate::tree::*;
    pub use crate::tree_ext::*;
    pub use crate::codegen::*;
    pub use crate::error::*;
    pub use crate::parser::*;
    pub use crate::syntax::*;
    pub const ZERO: crate::Zero = crate::Zero;
    pub const TRUE: crate::Bool = crate::Bool(true);
}

mod interface;
mod interface_id;

// ===== definitions =====

/// Create interface from protocol definition.
#[proc_macro]
pub fn interface(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, interface::impl_interface)
}

/// Create interface enum.
#[proc_macro]
pub fn interface_id(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, interface_id::impl_interface_id)
}

// ===== helpers =====

fn call(tokens: p::TokenStream, f: fn(Parser) -> Result<TokenStream, Error>) -> p::TokenStream {
    f(Parser::new(tokens.into())).map_or_else(<_>::into, <_>::into)
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

static KEYWORDS: [&str; 40] = [
    "as", "async", "await", "become", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type",
    "union", "unsafe", "use", "where", "while",
];

#[derive(Clone, Copy)]
struct Zero;

impl From<Zero> for tree::TokenTree {
    fn from(_: Zero) -> Self {
        tree::TokenTree::Literal(tree::Literal::u8_unsuffixed(0))
    }
}

#[derive(Clone, Copy)]
struct Bool(bool);

impl From<Bool> for tree::TokenTree {
    fn from(ok: Bool) -> Self {
        tree::TokenTree::Ident(tree::Ident::new(
            if ok.0 { "true" } else { "false" },
            Span::call_site(),
        ))
    }
}
