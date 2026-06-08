use proc_macro::{Group, Ident, Literal, Punct, TokenStream, TokenTree};

macro_rules! generate {
    () => {proc_macro::TokenStream::new()};
    ($($tt:tt)*) => {{
        let mut tokens = proc_macro::TokenStream::new();
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
            Group::new(proc_macro::Delimiter::Brace, crate::codegen::generate!($($gt)*)),
            &mut $tokens
        );
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};
    ($tokens:ident [ $($gt:tt)* ] $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens(
            Group::new(proc_macro::Delimiter::Bracket, crate::codegen::generate!($($gt)*)),
            &mut $tokens
        );
        crate::codegen::gen_extends!($tokens $($tt)*);
    }};
    ($tokens:ident ( $($gt:tt)* ) $($tt:tt)*) => {{
        crate::codegen::ToTokens::into_tokens(
            Group::new(proc_macro::Delimiter::Parenthesis, crate::codegen::generate!($($gt)*)),
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
                tokens.extend(Some(self));
            }
        }
    )*};
}
impl_single!(Ident, Punct, Group, Literal, TokenTree);

macro_rules! impl_iter {
    ($( $(const $n:ident)? $me:ty ),*) => {$(
        impl$(<const $n: usize>)? ToTokens for $me {
            fn into_tokens(self, tokens: &mut TokenStream) {
                tokens.extend(self);
            }
        }
    )*};
}
impl_iter!(TokenStream, const N [Punct; N], const N [TokenTree; N]);
