use crate::tree::*;

/// Returns and iterator of `TokenTree`.
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
    ($l:literal) => {Some(LiteralType::to_token_tree($l))};
    ($t:ident) => {Some(TokenTree::from(Ident::new(stringify!($t), Span::call_site())))};
}

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
    (#$i:ident) => {<_>::into_iter(Some(TokenTree::from($i.clone())))};
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

pub(crate) use {gentoken, token_stream, generate, generate as g, impl_generate};

pub trait LiteralType {
    fn to_token_tree(self) -> TokenTree;
}

impl LiteralType for bool {
    fn to_token_tree(self) -> TokenTree {
        Ident::new(if self { "true" } else { "false" }, Span::call_site()).into()
    }
}

impl LiteralType for usize {
    fn to_token_tree(self) -> TokenTree {
        Literal::usize_unsuffixed(self).into()
    }
}

impl LiteralType for &str {
    fn to_token_tree(self) -> TokenTree {
        Literal::string(self).into()
    }
}
