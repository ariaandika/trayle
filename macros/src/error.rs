use crate::tree::*;
use crate::span::*;

// ===== trait =====

pub trait Expect {
    const EXPECT: &str;

    fn expected(self) -> Error
    where
        Self: Spanned + Sized,
    {
        Error::new(format!("expected `{}`", Self::EXPECT), self)
    }
}

// ===== Error =====

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Error {
    msg: Literal,
    span: Span,
}

impl Error {
    pub fn new<M: AsRef<str>, S: Spanned>(msg: M, span: S) -> Self {
        Self {
            msg: Literal::string(msg.as_ref()),
            span: span.span(),
        }
    }

    pub fn new_site<M: AsRef<str>>(msg: M) -> Self {
        Self::new(msg, Span::call_site())
    }

    fn generate(&self) -> impl Iterator<Item = TokenTree> + Clone {
        [
            Ident::new("compile_error", self.span).into(),
            Punct::new('!', Spacing::Alone).into(),
            Group::new(
                Delimiter::Parenthesis,
                <_>::from_iter(Some(TokenTree::from(self.msg.clone()))),
            )
            .spanned(self.span)
            .into(),
            Punct::new(';', Spacing::Alone).into(),
        ]
        .into_iter()
    }
}

impl From<Error> for TokenStream {
    fn from(value: Error) -> Self {
        value.generate().collect()
    }
}

impl From<Error> for p::TokenStream {
    fn from(value: Error) -> Self {
        TokenStream::from(value).into()
    }
}
