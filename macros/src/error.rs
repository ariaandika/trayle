use crate::tree::*;
use crate::codegen::*;

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

    fn generate(&self) -> impl Iterator<Item = TokenTree> {
        [
            Ident::new("compile_error", self.span).into(),
            Punct::new('!', Spacing::Alone).into(),
            Group::new(
                Delimiter::Parenthesis,
                TokenTree::Literal(Literal::string(&self.msg)).into_token_stream(),
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

impl ToTokens for crate::Error {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.generate());
    }
}
