//! The `TokenTree` and its tree types are an absolute introvert, one cannot do basic logic without
//! reparsing the `ToString` implementation.
//!
//! These types wrap them and give common functionality and if needed, cache it locally.

use std::cell::OnceCell;
use proc_macro as p;

pub use proc_macro::{Group, Punct, Literal, Span, Delimiter, Spacing};

pub trait TokenResult {
    fn into_token_stream(self) -> p::TokenStream;
}

impl TokenResult for Result<TokenStream, crate::Error> {
    fn into_token_stream(self) -> p::TokenStream {
        match self {
            Ok(ok) => ok.into(),
            Err(err) => TokenStream::from(err).into(),
        }
    }
}

// ===== TokenStream =====

#[derive(Clone)]
pub struct TokenStream(p::TokenStream);

impl TokenStream {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn push<T: Into<TokenTree>>(&mut self, value: T) {
        self.0.extend(Some(p::TokenTree::from(value.into())));
    }
}

impl TokenStream {
    pub fn new() -> Self {
        Self(p::TokenStream::new())
    }
}

pub struct IntoIter(std::iter::Map<p::token_stream::IntoIter, fn(p::TokenTree) -> TokenTree>);

// ===== TokenTree =====

#[derive(Debug)]
pub enum TokenTree {
    Group(p::Group),
    Ident(Ident),
    Punct(p::Punct),
    Literal(p::Literal),
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

    pub fn new_string(string: String, span: Span) -> Ident {
        Self {
            token: p::Ident::new(&string, span),
            string: OnceCell::from(string),
        }
    }

    pub fn as_str(&self) -> &str {
        self.string.get_or_init(|| self.token.to_string())
    }

    pub fn span(&self) -> Span {
        self.token.span()
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
            p::TokenTree::Group(g) => Self::Group(g),
            p::TokenTree::Ident(i) => Self::Ident(i.into()),
            p::TokenTree::Punct(p) => Self::Punct(p),
            p::TokenTree::Literal(l) => Self::Literal(l),
        }
    }
}

impl From<TokenTree> for p::TokenTree {
    fn from(value: TokenTree) -> Self {
        match value {
            TokenTree::Group(g) => p::TokenTree::Group(g),
            TokenTree::Ident(i) => p::TokenTree::Ident(i.into()),
            TokenTree::Punct(p) => p::TokenTree::Punct(p),
            TokenTree::Literal(l) => p::TokenTree::Literal(l),
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
token_tree_from!(Group(p::Group), Ident(Ident), Punct(p::Punct), Literal(p::Literal));


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
