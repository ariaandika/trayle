use proc_macro::TokenStream;
use error::Error;
use parser::Parser;

mod parser;
mod syntax;
mod codegen;
mod error;

// ===== implementations =====

mod prelude {
    pub(crate) use super::{to_camel, to_snake};
    pub use crate::codegen::*;
    pub use crate::error::Error;
    pub use crate::parser::*;
    pub use crate::syntax::*;
    pub use proc_macro::{Delimiter, Spacing};
    pub use proc_macro::{Group, Ident, Literal, Punct, Span, TokenStream, TokenTree};
}

mod interface;
mod opcode;
mod message;
mod wl_enum;
mod bitfield;
mod protocol;

// ===== exports =====

/// Implement `FromObjectId`, `AsObjectId` and `AsInterface`.
#[proc_macro_derive(Interface)]
pub fn interface(tokens: TokenStream) -> TokenStream {
    interface::process(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

/// Implement `OpCode` and `Display`.
#[proc_macro_derive(OpCode)]
pub fn op_code(tokens: TokenStream) -> TokenStream {
    opcode::process(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

/// Implement `Decode`, `Encode`, `AsInterface`, and add constructor of the message in the interface
/// object.
#[proc_macro_derive(Message, attributes(request, event, fd))]
pub fn message(tokens: TokenStream) -> TokenStream {
    message::process(Parser::new(tokens)).unwrap_or_else(<_>::into)
}

/// Implement `WlEnum` and `Display`.
#[proc_macro_derive(WlEnum)]
pub fn wl_enum(tokens: TokenStream) -> TokenStream {
    wl_enum::process(Parser::new(tokens)).unwrap_or_else(<_>::into)
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
    protocol::process(Parser::new(tokens)).unwrap_or_else(<_>::into)
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
pub fn bitfield(tokens: TokenStream) -> TokenStream {
    bitfield::process(Parser::new(tokens)).unwrap_or_else(<_>::into)
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
