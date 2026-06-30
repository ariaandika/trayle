use proc_macro::Span;

pub trait Spanned {
    fn span(&self) -> Span;

    fn set_span(&mut self, span: Span);

    fn spanned(mut self, span: Span) -> Self
    where
        Self: Sized,
    {
        self.set_span(span);
        self
    }
}

impl Spanned for Span {
    fn span(&self) -> Span {
        *self
    }

    fn set_span(&mut self, span: Span) {
        *self = span;
    }
}
