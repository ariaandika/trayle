use std::iter::Flatten;
use std::option::IntoIter as OptIter;

use crate::span::*;
use crate::tree::*;

// ===== Either =====

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Iterator, R: Iterator<Item = L::Item>> Iterator for Either<L, R> {
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Either::Left(l) => l.size_hint(),
            Either::Right(r) => r.size_hint(),
        }
    }
}

// ===== bool ext =====

pub trait BoolExt: Sized {
    fn then_stream<I, F>(self, f: F) -> Flatten<OptIter<I::IntoIter>>
    where
        I: IntoIterator<Item = TokenTree>,
        F: FnOnce() -> I;
}

impl BoolExt for bool {
    fn then_stream<I, F>(self, f: F) -> Flatten<OptIter<I::IntoIter>>
    where
        I: IntoIterator<Item = TokenTree>,
        F: FnOnce() -> I,
    {
        self.then(|| f().into_iter()).into_iter().flatten()
    }
}

// ===== option ext =====

pub trait OptionExt<T> {
    fn map_stream<I, F>(self, f: F) -> Flatten<OptIter<I>>
    where
        I: Iterator<Item = TokenTree>,
        F: FnOnce(T) -> I;
}

impl<T> OptionExt<T> for Option<T> {
    fn map_stream<I, F>(self, f: F) -> Flatten<OptIter<I>>
    where
        I: Iterator<Item = TokenTree>,
        F: FnOnce(T) -> I,
    {
        self.map(f).into_iter().flatten()
    }
}

// ===== iterator ext =====

pub trait IteratorExt: Sized + Iterator<Item = TokenTree> {
    fn left<R>(self) -> Either<Self, R> {
        Either::Left(self)
    }

    fn right<L>(self) -> Either<L, Self> {
        Either::Right(self)
    }
}

impl<I: Iterator<Item = TokenTree>> IteratorExt for I {}

// ===== case conversion =====

impl Ident {
    pub(crate) fn to_camel(&self) -> Ident {
        Self::new_string(to_camel(self.as_str()), self.span())
    }

    pub(crate) fn to_snake(&self) -> Ident {
        Self::new_string(to_snake(self.as_str()), self.span())
    }

    pub(crate) fn to_lit_snake(&self) -> Literal {
        Literal::string(&to_snake(self.as_str())).spanned(self.span())
    }
}

pub fn to_camel(string: &str) -> String {
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

pub fn to_snake(string: &str) -> String {
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

// ===== other =====

pub static KEYWORDS: [&str; 40] = [
    "as", "async", "await", "become", "break", "const", "continue", "crate", "dyn", "else", "enum",
    "extern", "false", "fn", "for", "gen", "if", "impl", "in", "let", "loop", "match", "mod", "move",
    "mut", "pub", "ref", "return", "self", "static", "struct", "super", "trait", "true", "type",
    "union", "unsafe", "use", "where", "while",
];

pub const ZERO: Zero = Zero;

#[derive(Clone, Copy)]
pub struct Zero;

impl From<Zero> for TokenTree {
    fn from(_: Zero) -> Self {
        TokenTree::Literal(Literal::u8_unsuffixed(0))
    }
}

pub const TRUE: Bool = Bool(true);

#[derive(Clone, Copy)]
pub struct Bool(pub bool);

impl From<Bool> for TokenTree {
    fn from(ok: Bool) -> Self {
        TokenTree::Ident(Ident::new(
            if ok.0 { "true" } else { "false" },
            Span::call_site(),
        ))
    }
}
