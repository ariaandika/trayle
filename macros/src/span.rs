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

    fn unspan(&mut self) -> Span
    where
        Self: Sized,
    {
        let span = self.span();
        self.set_span(Span::call_site());
        span
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
