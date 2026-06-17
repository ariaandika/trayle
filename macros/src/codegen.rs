use crate::tree::*;

macro_rules! token_stream {
    () => { TokenStream::new() };
    ($($tt:tt)*) => { crate::codegen::generate!($($tt)*).collect::<TokenStream>() };
}

macro_rules! generate {
    () => {IntoIterator::into_iter(None::<TokenTree>)};
    ($($tt:tt)*) => {
        crate::codegen::impl_generate!($($tt)*)
    };
}

macro_rules! impl_generate {

    // arbitrary input
    (#$i:ident) => {
        IntoIterator::into_iter(Some(TokenTree::from($i.clone())))
    };
    (#$i:ident $($tt:tt)*) => {
        IntoIterator::into_iter(Some(TokenTree::from($i.clone())))
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    // arbitrary blocked input
    (#{ $e:expr }) => {
        IntoIterator::into_iter(Some(TokenTree::from($e)))
    };
    (#{ $e:expr } $($tt:tt)*) => {
        IntoIterator::into_iter(Some(TokenTree::from($e)))
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    // optional arbitrary input
    (?$i:ident) => {
        IntoIterator::into_iter($i.clone().map(TokenTree::from))
    };
    (?$i:ident $($tt:tt)*) => {
        IntoIterator::into_iter($i.clone().map(TokenTree::from))
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    // iterator input
    (@$i:ident) => {
        IntoIterator::into_iter($i.clone())
    };
    (@$i:ident $($tt:tt)*) => {
        IntoIterator::into_iter($i.clone())
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    // groups
    ({ $($gt:tt)* }) => {
        IntoIterator::into_iter(Some(TokenTree::from(Group::new(Delimiter::Brace, crate::codegen::generate!($($gt)*).collect()))))
    };
    ({ $($gt:tt)* } $($tt:tt)*) => {
        IntoIterator::into_iter(Some(TokenTree::from(Group::new(Delimiter::Brace, crate::codegen::generate!($($gt)*).collect()))))
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    ([ $($gt:tt)* ]) => {
        IntoIterator::into_iter(Some(TokenTree::from(Group::new(Delimiter::Bracket, crate::codegen::generate!($($gt)*).collect()))))
    };
    ([ $($gt:tt)* ] $($tt:tt)*) => {
        IntoIterator::into_iter(Some(TokenTree::from(Group::new(Delimiter::Bracket, crate::codegen::generate!($($gt)*).collect()))))
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    (( $($gt:tt)* )) => {
        IntoIterator::into_iter(Some(TokenTree::from(Group::new(Delimiter::Parenthesis, crate::codegen::generate!($($gt)*).collect()))))
    };
    (( $($gt:tt)* ) $($tt:tt)*) => {
        IntoIterator::into_iter(Some(TokenTree::from(Group::new(Delimiter::Parenthesis, crate::codegen::generate!($($gt)*).collect()))))
            .chain(crate::codegen::impl_generate!($($tt)*))
    };

    // else
    ($t:tt) => {
        IntoIterator::into_iter(crate::codegen::gentoken!($t))
    };
    ($t:tt $($tt:tt)*) => {
        IntoIterator::into_iter(crate::codegen::gentoken!($t)).chain(crate::codegen::impl_generate!($($tt)*))
    };
}

macro_rules! gentoken {
    (::) => {[TokenTree::from(Punct::new(':', Spacing::Joint)), TokenTree::from(Punct::new(':', Spacing::Joint))]};
    (==) => {[TokenTree::from(Punct::new('=', Spacing::Joint)), TokenTree::from(Punct::new('=', Spacing::Joint))]};
    (=>) => {[TokenTree::from(Punct::new('=', Spacing::Joint)), TokenTree::from(Punct::new('>', Spacing::Joint))]};
    (<=) => {[TokenTree::from(Punct::new('<', Spacing::Joint)), TokenTree::from(Punct::new('=', Spacing::Joint))]};
    (>=) => {[TokenTree::from(Punct::new('>', Spacing::Joint)), TokenTree::from(Punct::new('=', Spacing::Joint))]};
    (>>) => {[TokenTree::from(Punct::new('>', Spacing::Joint)), TokenTree::from(Punct::new('>', Spacing::Joint))]};
    (<<) => {[TokenTree::from(Punct::new('<', Spacing::Joint)), TokenTree::from(Punct::new('<', Spacing::Joint))]};
    (->) => {[TokenTree::from(Punct::new('-', Spacing::Joint)), TokenTree::from(Punct::new('>', Spacing::Joint))]};
    (_) => {Some(TokenTree::from(Ident::new("_", Span::call_site())))};
    (=) => {Some(TokenTree::from(Punct::new('=', Spacing::Alone)))};
    (<) => {Some(TokenTree::from(Punct::new('<', Spacing::Alone)))};
    (>) => {Some(TokenTree::from(Punct::new('>', Spacing::Alone)))};
    (!) => {Some(TokenTree::from(Punct::new('!', Spacing::Alone)))};
    (~) => {Some(TokenTree::from(Punct::new('~', Spacing::Alone)))};
    (+) => {Some(TokenTree::from(Punct::new('+', Spacing::Alone)))};
    (-) => {Some(TokenTree::from(Punct::new('-', Spacing::Alone)))};
    (*) => {Some(TokenTree::from(Punct::new('*', Spacing::Alone)))};
    (/) => {Some(TokenTree::from(Punct::new('/', Spacing::Alone)))};
    (%) => {Some(TokenTree::from(Punct::new('%', Spacing::Alone)))};
    (^) => {Some(TokenTree::from(Punct::new('^', Spacing::Alone)))};
    (&) => {Some(TokenTree::from(Punct::new('&', Spacing::Alone)))};
    (|) => {Some(TokenTree::from(Punct::new('|', Spacing::Alone)))};
    (@) => {Some(TokenTree::from(Punct::new('@', Spacing::Alone)))};
    (.) => {Some(TokenTree::from(Punct::new('.', Spacing::Alone)))};
    (,) => {Some(TokenTree::from(Punct::new(',', Spacing::Alone)))};
    (;) => {Some(TokenTree::from(Punct::new(';', Spacing::Alone)))};
    (:) => {Some(TokenTree::from(Punct::new(':', Spacing::Alone)))};
    (#) => {Some(TokenTree::from(Punct::new('#', Spacing::Alone)))};
    (?) => {Some(TokenTree::from(Punct::new('?', Spacing::Alone)))};
    ($lf:lifetime) => {[
        TokenTree::from(Punct::new('\'', Spacing::Joint)),
        TokenTree::from(Ident::new(&stringify!($lf)[1..], Span::call_site())),
    ]};
    ($l:literal) => {Some(TokenTree::from(Literal::string($l)))};
    ($t:ident) => {Some(TokenTree::from(Ident::new(stringify!($t), Span::call_site())))};
}

pub(crate) use {token_stream, gentoken, impl_generate, generate};

// ===== ToTokens =====

#[allow(unused)] // temp
pub trait ToTokens {
    fn to_tokens(&self, tokens: &mut TokenStream);

    fn to_token_stream(&self) -> TokenStream {
        let mut stream = TokenStream::new();
        self.to_tokens(&mut stream);
        stream
    }

    // fn into_token_stream(self) -> TokenStream
    // where
    //     Self: Sized,
    // {
    //     self.to_token_stream()
    // }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        if let Some(me) = self {
            me.to_tokens(tokens);
        }
    }
}

impl ToTokens for TokenStream {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.clone());
    }
}

macro_rules! impl_single {
    ($($me:ident),*) => {$(
        impl ToTokens for $me {
            fn to_tokens(&self, tokens: &mut TokenStream) {
                tokens.push(self.clone());
            }
        }
    )*};
}
impl_single!(Ident, Punct, Group, Literal, TokenTree);
