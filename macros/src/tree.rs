//! The `TokenTree` and its tree types are an absolute introvert, one cannot do basic logic without
//! reparsing the `ToString` implementation.
//!
//! These types wrap them and give common functionality and if needed, cache it locally.

use std::cell::OnceCell;

pub use proc_macro as p;
pub use proc_macro::{Span, Delimiter, Spacing};

// ===== TokenStream =====

#[derive(Default, Clone)]
pub struct TokenStream(p::TokenStream);

impl TokenStream {
    pub fn new() -> Self {
        Self(p::TokenStream::new())
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push<T: Into<TokenTree>>(&mut self, value: T) {
        self.0.extend(Some(p::TokenTree::from(value.into())));
    }
}

pub struct IntoIter(std::iter::Map<p::token_stream::IntoIter, fn(p::TokenTree) -> TokenTree>);

// ===== TokenTree =====

#[derive(Debug, Clone)]
pub enum TokenTree {
    Group(Group),
    Ident(Ident),
    Punct(Punct),
    Literal(Literal),
}

impl TokenTree {
    pub fn span(&self) -> Span {
        match self {
            TokenTree::Group(g) => g.span(),
            TokenTree::Ident(i) => i.span(),
            TokenTree::Punct(p) => p.span(),
            TokenTree::Literal(l) => l.span(),
        }
    }
}

// ===== Group =====

#[derive(Debug, Clone)]
pub struct Group(p::Group);

impl Group {
    pub fn new(delim: Delimiter, stream: TokenStream) -> Self {
        Self(p::Group::new(delim, stream.into()))
    }

    pub fn body_parser(&self) -> crate::Parser {
        crate::Parser::new(self.0.stream().into())
    }

    pub fn stream(&self) -> TokenStream {
        self.0.stream().into()
    }
}

// ===== Ident =====

#[derive(Clone)]
pub struct Ident {
    token: p::Ident,
    string: OnceCell<String>,
}

impl Ident {
    pub fn new(string: &str, span: Span) -> Ident {
        Self {
            token: p::Ident::new(string, span),
            string: OnceCell::new(),
        }
    }

    pub fn new_raw(string: &str, span: Span) -> Ident {
        Self {
            token: p::Ident::new_raw(string, span),
            string: OnceCell::new(),
        }
    }

    pub fn new_string(string: String, span: Span) -> Ident {
        Self {
            token: p::Ident::new(&string, span),
            string: OnceCell::from(string),
        }
    }

    pub fn as_str(&self) -> &str {
        self.string.get_or_init(|| self.token.to_string())
    }

    pub fn set_span(&mut self, span: Span) {
        self.token.set_span(span);
    }

    pub fn span(&self) -> Span {
        self.token.span()
    }
}

// ===== Punct =====

#[derive(Debug, Clone)]
pub struct Punct(p::Punct);

impl Punct {
    pub fn new(ch: char, spacing: Spacing) -> Self {
        Self(p::Punct::new(ch, spacing))
    }
}

// ===== Literal =====

#[derive(Debug, Clone)]
pub struct Literal(p::Literal);

impl Literal {
    pub fn u8_unsuffixed(n: u8) -> Self {
        Self(p::Literal::u8_unsuffixed(n))
    }

    pub fn u16_suffixed(n: u16) -> Self {
        Self(p::Literal::u16_suffixed(n))
    }

    pub fn u32_unsuffixed(n: u32) -> Self {
        Self(p::Literal::u32_unsuffixed(n))
    }

    pub fn usize_unsuffixed(n: usize) -> Self {
        Self(p::Literal::usize_unsuffixed(n))
    }

    pub fn character(ch: char) -> Self {
        Self(p::Literal::character(ch))
    }

    pub fn string(string: &str) -> Self {
        Self(p::Literal::string(string))
    }
}

// ===== trait TokenStream =====

impl From<p::TokenStream> for TokenStream {
    fn from(value: p::TokenStream) -> Self { Self(value) }
}
impl From<TokenStream> for p::TokenStream {
    fn from(value: TokenStream) -> Self { value.0 }
}
impl Iterator for IntoIter {
    type Item = TokenTree;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}
impl Extend<TokenTree> for TokenStream {
    fn extend<T: IntoIterator<Item = TokenTree>>(&mut self, iter: T) {
        self.0.extend(iter.into_iter().map(p::TokenTree::from));
    }
}
impl IntoIterator for TokenStream {
    type Item = TokenTree;
    type IntoIter = IntoIter;
    fn into_iter(self) -> Self::IntoIter {
        IntoIter(self.0.into_iter().map(<_>::into))
    }
}

// ===== trait tokens =====

impl FromIterator<TokenTree> for p::TokenStream {
    fn from_iter<T: IntoIterator<Item = TokenTree>>(iter: T) -> Self {
        <_>::from_iter(iter.into_iter().map(p::TokenTree::from))
    }
}

impl FromIterator<TokenTree> for TokenStream {
    fn from_iter<T: IntoIterator<Item = TokenTree>>(iter: T) -> Self {
        Self(<_>::from_iter(iter.into_iter().map(p::TokenTree::from)))
    }
}

impl From<p::TokenTree> for TokenTree {
    fn from(value: p::TokenTree) -> Self {
        match value {
            p::TokenTree::Group(g) => Self::Group(g.into()),
            p::TokenTree::Ident(i) => Self::Ident(i.into()),
            p::TokenTree::Punct(p) => Self::Punct(p.into()),
            p::TokenTree::Literal(l) => Self::Literal(l.into()),
        }
    }
}

impl From<TokenTree> for p::TokenTree {
    fn from(value: TokenTree) -> Self {
        match value {
            TokenTree::Group(g) => p::TokenTree::Group(g.into()),
            TokenTree::Ident(i) => p::TokenTree::Ident(i.into()),
            TokenTree::Punct(p) => p::TokenTree::Punct(p.into()),
            TokenTree::Literal(l) => p::TokenTree::Literal(l.into()),
        }
    }
}

macro_rules! token_tree_from {
    ($($v:ident($ty:ty)),*) => {
        $(
        impl From<$ty> for TokenTree {
            fn from(value: $ty) -> Self {
                Self::$v(value)
            }
        }
        )*
    };
}
token_tree_from!(Group(Group), Ident(Ident), Punct(Punct), Literal(Literal));

macro_rules! wrapper {
    ($($me:ident),*) => {
        $(
        impl From<p::$me> for $me {
            fn from(value: p::$me) -> Self {
                Self(value)
            }
        }
        impl From<$me> for p::$me {
            fn from(value: $me) -> Self {
                value.0
            }
        }
        impl std::ops::Deref for $me {
            type Target = p::$me;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl std::ops::DerefMut for $me {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut self.0
            }
        }
        )*
    };
}
wrapper!(Group, Punct, Literal);

impl From<p::Ident> for Ident {
    fn from(value: p::Ident) -> Self {
        Self {
            token: value,
            string: OnceCell::new(),
        }
    }
}

impl From<Ident> for p::Ident {
    fn from(value: Ident) -> Self {
        value.token
    }
}

impl std::fmt::Debug for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.token.fmt(f)
    }
}

impl std::fmt::Display for Ident {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.as_str().fmt(f)
    }
}

// ===== Extension =====

pub fn stream_if<I: IntoIterator<Item = TokenTree>, F: FnOnce() -> I>(
    cond: bool,
    f: F,
) -> std::iter::Flatten<std::option::IntoIter<I::IntoIter>> {
    if cond { Some(f().into_iter()) } else { None }
        .into_iter()
        .flatten()
}

pub trait OptionExt<T> {
    fn map_stream<I: Iterator<Item = TokenTree>, F: FnOnce(T) -> I>(
        self,
        f: F,
    ) -> std::iter::Flatten<std::option::IntoIter<I>>;
}

impl<T> OptionExt<T> for Option<T> {
    fn map_stream<I: Iterator<Item = TokenTree>, F: FnOnce(T) -> I>(
        self,
        f: F,
    ) -> std::iter::Flatten<std::option::IntoIter<I>> {
        self.map(f).into_iter().flatten()
    }
}

pub trait IteratorExt: Sized + Iterator<Item = TokenTree> {
    fn map_group(self, delim: Delimiter) -> impl Iterator<Item = TokenTree> {
        Some(Group::new(delim, self.collect()).into()).into_iter()
    }

    fn chain_back(
        self,
        other: impl Iterator<Item = TokenTree>,
    ) -> impl Iterator<Item = TokenTree> {
        other.chain(self)
    }
}

impl<I: Iterator<Item = TokenTree>> IteratorExt for I {}
