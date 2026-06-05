use proc_macro::*;

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
    (::) => {[
        proc_macro::Punct::new(':', Spacing::Joint),
        proc_macro::Punct::new(':', Spacing::Joint),
    ]};
    (==) => {[
        proc_macro::Punct::new('=', Spacing::Joint),
        proc_macro::Punct::new('=', Spacing::Joint),
    ]};
    (=>) => {[
        proc_macro::Punct::new('=', Spacing::Joint),
        proc_macro::Punct::new('>', Spacing::Joint),
    ]};
    (<=) => {[
        proc_macro::Punct::new('<', Spacing::Joint),
        proc_macro::Punct::new('=', Spacing::Joint),
    ]};
    (>=) => {[
        proc_macro::Punct::new('>', Spacing::Joint),
        proc_macro::Punct::new('=', Spacing::Joint),
    ]};
    (->) => {[
        proc_macro::Punct::new('-', Spacing::Joint),
        proc_macro::Punct::new('>', Spacing::Joint),
    ]};
    (_) => {proc_macro::Ident::new("_", Span::call_site())};
    (=) => {proc_macro::Punct::new('=', Spacing::Alone)};
    (<) => {proc_macro::Punct::new('<', Spacing::Alone)};
    (>) => {proc_macro::Punct::new('>', Spacing::Alone)};
    (!) => {proc_macro::Punct::new('!', Spacing::Alone)};
    (~) => {proc_macro::Punct::new('~', Spacing::Alone)};
    (+) => {proc_macro::Punct::new('+', Spacing::Alone)};
    (-) => {proc_macro::Punct::new('-', Spacing::Alone)};
    (*) => {proc_macro::Punct::new('*', Spacing::Alone)};
    (/) => {proc_macro::Punct::new('/', Spacing::Alone)};
    (%) => {proc_macro::Punct::new('%', Spacing::Alone)};
    (^) => {proc_macro::Punct::new('^', Spacing::Alone)};
    (&) => {proc_macro::Punct::new('&', Spacing::Alone)};
    (|) => {proc_macro::Punct::new('|', Spacing::Alone)};
    (@) => {proc_macro::Punct::new('@', Spacing::Alone)};
    (.) => {proc_macro::Punct::new('.', Spacing::Alone)};
    (,) => {proc_macro::Punct::new(',', Spacing::Alone)};
    (;) => {proc_macro::Punct::new(';', Spacing::Alone)};
    (:) => {proc_macro::Punct::new(':', Spacing::Alone)};
    (#) => {proc_macro::Punct::new('#', Spacing::Alone)};
    (?) => {proc_macro::Punct::new('?', Spacing::Alone)};
    ($l:literal) => {proc_macro::Literal::new(stringify!($l), proc_macro::Span::call_site())};
    ($t:ident) => {proc_macro::Ident::new(stringify!($t), proc_macro::Span::call_site())};
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

    // fn into_token_stream(self) -> TokenStream {
    //     let mut tokens = TokenStream::new();
    //     self.into_tokens(&mut tokens);
    //     tokens
    // }
}

impl ToTokens for Ident {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend([self]);
    }
}

impl ToTokens for Punct {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend([self]);
    }
}

impl ToTokens for [Punct; 2] {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend(self);
    }
}

impl ToTokens for Group {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend([self]);
    }
}

impl ToTokens for Literal {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend([self]);
    }
}

impl ToTokens for TokenStream {
    fn into_tokens(self, tokens: &mut TokenStream) {
        tokens.extend(self);
    }
}
