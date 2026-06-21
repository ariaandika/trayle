use proc_macro::Span;
use tree::{TokenStream, p};
use error::Error;
use parser::Parser;

mod tree;
mod codegen;
mod error;
mod parser;

// ===== syntax =====

mod syntax;
mod attr;

// ===== implementations =====

mod prelude {
    pub(crate) use super::{Bool, KEYWORDS, to_camel, to_snake};
    pub use crate::tree::*;
    pub use crate::codegen::*;
    pub use crate::error::*;
    pub use crate::parser::*;
    pub use crate::syntax::*;
    pub use crate::attr::*;
    pub const ZERO: crate::Zero = crate::Zero;
    pub const TRUE: crate::Bool = crate::Bool(true);
}

mod interface;
mod opcode;
mod message;
mod wl_enum;
mod bitfield;
mod protocol;

// ===== definitions =====

/// Implement `FromObjectId`, `AsObjectId` and `AsInterface`.
///
/// Attributes: `#[interface(global = <Ident>, data = <Ident>)]`.
///
/// Both attribute are optional.
#[proc_macro_derive(Interface, attributes(interface))]
pub fn interface(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, interface::process)
}

/// Implement `OpCode` and `Display`.
#[proc_macro_derive(OpCode)]
pub fn opcode(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, opcode::process)
}

/// Implement `Decode`, `Encode`, `AsInterface`, `WlMessage` and add constructor of the message in
/// the interface object.
///
/// Attributes: `#[message(request = <Ident>, event = <Ident>, destructor)]`
///
/// Either `request` or `event` are required.
///
/// Optionally, add `destructor` to mark operation as `type=destructor`, it will set
/// `Operator::IS_DESTRUCTOR`  to `true`.
#[proc_macro_derive(Message, attributes(message, fd))]
pub fn message(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, message::process)
}

/// Implement `WlEnum`, `fmt::Display`, `display::Display2`.
///
/// This also add name getter.
#[proc_macro_derive(WlEnum)]
pub fn wl_enum(tokens:p::TokenStream) -> p::TokenStream {
    call(tokens, wl_enum::process)
}

/// Implement `WlEnum`, `Display`, `Bit{And, Or, Xor}` and define constant for each entries.
///
/// Target struct must be a single field struct of `u32`.
///
/// Currently, the `WlEnum` implementation ignore unknown bits.
///
/// ```ignore
/// bitfield! {
///     DndAction;
///
///     None = 0,
///     Copy = 1,
///     Move = 2,
///     Ask = 4,
/// }
/// ```
#[proc_macro]
pub fn bitfield(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, bitfield::process)
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
pub fn protocol(tokens: p::TokenStream) -> p::TokenStream {
    call(tokens, protocol::process)
}

fn call(tokens: p::TokenStream, f: fn(Parser) -> Result<TokenStream, Error>) -> p::TokenStream {
    codegen::ToTokens::into_token_stream(f(Parser::new(tokens.into()))).into()
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
