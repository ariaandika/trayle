use crate::tree::*;

macro_rules! generate {
    () => {TokenStream::new()};
    ($($tt:tt)*) => {{
        let mut tokens = TokenStream::new();
        crate::codegen::gen_extends!(tokens $($tt)*);
        tokens
    }};
}

macro_rules! gen_extends {

    // arbitrary input
    ($tokens:ident #&$i:ident $($tt:tt)*) => {{
        crate::codegen::ToTokens::to_tokens(&$i, &mut $tokens);
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};
    ($tokens:ident #$i:ident $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens($i, &mut $tokens);
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};

    // groups
    ($tokens:ident { $($gt:tt)* } $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens(
            Group::new(Delimiter::Brace, crate::codegen::generate!($($gt)*).into()),
            &mut $tokens
        );
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};
    ($tokens:ident [ $($gt:tt)* ] $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens(
            Group::new(Delimiter::Bracket, crate::codegen::generate!($($gt)*).into()),
            &mut $tokens
        );
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};
    ($tokens:ident ( $($gt:tt)* ) $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens(
            Group::new(Delimiter::Parenthesis, crate::codegen::generate!($($gt)*).into()),
            &mut $tokens
        );
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};

    // else
    ($tokens:ident $t:tt $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens(crate::codegen::gen_token!($t), &mut $tokens);
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};
    ($tokens:ident) => { };
}

macro_rules! gen_token {
    (::) => {[Punct::new(':', Spacing::Joint), Punct::new(':', Spacing::Joint)]};
    (==) => {[Punct::new('=', Spacing::Joint), Punct::new('=', Spacing::Joint)]};
    (=>) => {[Punct::new('=', Spacing::Joint), Punct::new('>', Spacing::Joint)]};
    (<=) => {[Punct::new('<', Spacing::Joint), Punct::new('=', Spacing::Joint)]};
    (>=) => {[Punct::new('>', Spacing::Joint), Punct::new('=', Spacing::Joint)]};
    (>>) => {[Punct::new('>', Spacing::Joint), Punct::new('>', Spacing::Joint)]};
    (<<) => {[Punct::new('<', Spacing::Joint), Punct::new('<', Spacing::Joint)]};
    (->) => {[Punct::new('-', Spacing::Joint), Punct::new('>', Spacing::Joint)]};
    (_) => {Ident::new("_", Span::call_site())};
    (=) => {Punct::new('=', Spacing::Alone)};
    (<) => {Punct::new('<', Spacing::Alone)};
    (>) => {Punct::new('>', Spacing::Alone)};
    (!) => {Punct::new('!', Spacing::Alone)};
    (~) => {Punct::new('~', Spacing::Alone)};
    (+) => {Punct::new('+', Spacing::Alone)};
    (-) => {Punct::new('-', Spacing::Alone)};
    (*) => {Punct::new('*', Spacing::Alone)};
    (/) => {Punct::new('/', Spacing::Alone)};
    (%) => {Punct::new('%', Spacing::Alone)};
    (^) => {Punct::new('^', Spacing::Alone)};
    (&) => {Punct::new('&', Spacing::Alone)};
    (|) => {Punct::new('|', Spacing::Alone)};
    (@) => {Punct::new('@', Spacing::Alone)};
    (.) => {Punct::new('.', Spacing::Alone)};
    (,) => {Punct::new(',', Spacing::Alone)};
    (;) => {Punct::new(';', Spacing::Alone)};
    (:) => {Punct::new(':', Spacing::Alone)};
    (#) => {Punct::new('#', Spacing::Alone)};
    (?) => {Punct::new('?', Spacing::Alone)};
    ($lf:lifetime) => {[
        TokenTree::from(Punct::new('\'', Spacing::Joint)),
        TokenTree::from(Ident::new(&stringify!($lf)[1..], Span::call_site())),
    ]};
    ($l:literal) => {Literal::string($l)};
    ($t:ident) => {Ident::new(stringify!($t), Span::call_site())};
    () => {};
}

pub(crate) use {gen_token, gen_extends, generate};

pub trait ToTokens: Sized {
    fn into_tokens(self, tokens: &mut TokenStream);

    fn to_tokens(&self, tokens: &mut TokenStream)
    where
        Self: Clone,
    {
        self.clone().into_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn into_tokens(self, tokens: &mut TokenStream) {
        if let Some(me) = self {
            me.into_tokens(tokens);
        }
    }
}

macro_rules! impl_single {
    ($($me:ident),*) => {$(
        impl ToTokens for $me {
            fn into_tokens(self, tokens: &mut TokenStream) {
                tokens.push(self);
            }
        }
    )*};
}
impl_single!(Ident, Punct, Group, Literal, TokenTree);

impl ToTokens for TokenStream {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend(self);
    }
}

impl<const N: usize> ToTokens for [Punct; N] {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend(self.into_iter().map(<_>::into));
    }
}

impl<const N: usize> ToTokens for [TokenTree; N] {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend(self);
    }
}
