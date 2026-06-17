use crate::prelude::*;

// ===== Error =====

pub struct Error {
    msg: String,
    span: Span,
}

impl Error {
    pub fn new<S: Into<String>>(msg: S) -> Self {
        Self { msg: msg.into(), span: Span::call_site() }
    }

    pub fn spanned<S: Into<String>>(msg: S, span: Span) -> Self {
        Self { msg: msg.into(), span }
    }

    pub fn eof() -> Error {
        Self {
            msg: "unexpected EOF".into(),
            span: Span::call_site(),
        }
    }

    pub fn context(self, cx: &str) -> Error {
        let Self { mut msg, span } = self;
        msg.insert_str(0, cx);
        Self { msg, span }
    }
}

impl From<Error> for TokenStream {
    fn from(value: Error) -> Self {
        <_>::from_iter([
            TokenTree::Ident(Ident::new("compile_error", value.span)),
            Punct::new('!', Spacing::Alone).into(),
            Group::new(
                Delimiter::Parenthesis,
                TokenStream::from_iter([TokenTree::Literal(Literal::string(&value.msg))]),
            ).into(),
            Punct::new(';', Spacing::Alone).into(),
        ])
    }
}
