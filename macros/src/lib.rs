#![allow(clippy::module_inception)]
use tree::{TokenStream, p};
use error::Result;
use parser::Parser;

mod span;
mod error;
mod tree;
mod ext;
mod codegen;
mod to_tokens;
mod parser;

// ===== syntax =====

mod syntax;

// ===== implementations =====

mod prelude {
    pub use crate::span::*;
    pub use crate::tree::*;
    pub use crate::ext::*;
    pub use crate::codegen::*;
    pub use crate::error::*;
    pub use crate::parser::*;
    pub use crate::syntax::*;
}

mod interface;
mod interface_id;

// ===== helpers =====

fn call(tokens: p::TokenStream, f: fn(Parser) -> Result<TokenStream>) -> p::TokenStream {
    f(Parser::new(tokens.into())).map_or_else(<_>::into, <_>::into)
}

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
