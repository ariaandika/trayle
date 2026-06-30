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

pub struct Error(Literal);

impl Spanned for Error {
    fn span(&self) -> Span {
        self.0.span()
    }

    fn set_span(&mut self, span: Span) {
        self.0.set_span(span);
    }
}

impl Error {
    pub fn new<M: AsRef<str>, S: Spanned>(msg: M, span: S) -> Self {
        Self(Literal::string(msg.as_ref()).spanned(span.span()))
    }

    pub fn new_site<M: AsRef<str>>(msg: M) -> Self {
        Self::new(msg, Span::call_site())
    }

    fn generate(&self) -> impl Iterator<Item = TokenTree> + Clone {
        [
            Ident::new("compile_error", self.span()).into(),
            Punct::new('!', Spacing::Alone).into(),
            Group::new(
                Delimiter::Parenthesis,
                <_>::from_iter(Some(TokenTree::from(self.0.clone()))),
            )
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
