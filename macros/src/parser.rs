use crate::tree::*;
use crate::error::*;

// ===== trait =====

pub trait Parse: Sized {
    fn parse(parser: &mut Parser) -> Result<Self>;
}

impl Parse for Ident {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.token()
    }
}

impl Parse for Literal {
    fn parse(parser: &mut Parser) -> Result<Self> {
        parser.token()
    }
}

// ===== ParserInner =====

struct ParserInner {
    iter: IntoIter,
    cache: [Option<TokenTree>; 2],
}

impl ParserInner {
    fn new(token: TokenStream) -> Self {
        Self {
            iter: token.into_iter(),
            cache: [const { None }; _],
        }
    }

    fn next(&mut self) -> Option<TokenTree> {
        match self.cache[0].take() {
            Some(cached) => {
                self.cache[0] = self.cache[1].take();
                Some(cached)
            }
            None => self.iter.next(),
        }
    }

    fn peek(&mut self) -> Option<&TokenTree> {
        if self.cache[0].is_none() {
            self.cache[0] = self.iter.next();
        }
        self.cache[0].as_ref()
    }

    fn peek2(&mut self) -> Option<&TokenTree> {
        if self.cache[0].is_none() {
            self.cache[0] = self.iter.next();
        }
        if self.cache[1].is_none() {
            self.cache[1] = self.iter.next();
        }
        self.cache[1].as_ref()
    }

    fn next_if_map<O, F>(&mut self, f: F) -> Option<O>
    where
        F: FnOnce(TokenTree) -> Result<O, TokenTree>,
    {
        let tree = self.cache[0].take().or_else(|| self.iter.next())?;
        match f(tree) {
            Ok(ok) => {
                self.cache[0] = self.cache[1].take();
                Some(ok)
            }
            Err(tree) => {
                self.cache[0] = Some(tree);
                None
            }
        }
    }
}

// ===== Parser =====

pub struct Parser(ParserInner);

impl Parser {
    pub fn new(token: TokenStream) -> Self {
        Self(ParserInner::new(token))
    }

    pub fn next(&mut self) -> Option<TokenTree> {
        self.0.next()
    }

    pub fn peek(&mut self) -> Option<&TokenTree> {
        self.0.peek()
    }

    pub fn peek2(&mut self) -> Option<&TokenTree> {
        self.0.peek2()
    }

    pub fn next_if_map<O, F>(&mut self, f: F) -> Option<O>
    where
        F: FnOnce(TokenTree) -> Result<O, TokenTree>,
    {
        self.0.next_if_map(f)
    }

    pub fn try_next(&mut self) -> Result<TokenTree> {
        self.next().ok_or_else(|| Error::new_site("unexpected EOF"))
    }

    pub fn try_peek(&mut self) -> Result<&TokenTree> {
        self.peek().ok_or_else(|| Error::new_site("unexpected EOF"))
    }

    pub fn has_remaining(&mut self) -> bool {
        self.peek().is_some()
    }

    pub fn is_empty(&mut self) -> bool {
        self.peek().is_none()
    }

    pub fn drain(&mut self) -> TokenStream {
        std::iter::from_fn(|| self.next()).collect()
    }

    pub fn check_empty(&mut self) -> Result<()> {
        match self.next() {
            None => Ok(()),
            Some(t) => Err(Error::new(format!("leftover token: {}", t.found()), t)),
        }
    }

    pub fn call<T, F: FnOnce(&mut Parser) -> Result<T>>(&mut self, f: F) -> Result<T> {
        f(self)
    }

    pub fn parse<T: Parse>(&mut self) -> Result<T> {
        T::parse(self)
    }

    pub fn parse_full<T: Parse>(&mut self) -> Result<T> {
        let token = T::parse(self)?;
        match self.next() {
            None => Ok(token),
            Some(t) => Err(Error::new(format!("leftover token: {}", t.found()), t))
        }
    }
}

// TokenTree

pub trait Token: TryFrom<TokenTree, Error = TokenTree> + Expect { }

impl<T: TryFrom<TokenTree, Error = TokenTree> + Expect> Token for T { }

impl Parser {
    pub fn next_token<T: Token>(&mut self) -> Option<T> {
        self.next_if_map(<_>::try_into)
    }

    pub fn token<T: Token>(&mut self) -> Result<T> {
        self.try_next()?.try_into().map_err(|t: TokenTree| {
            Error::new(format!("expected `{}`, found: `{}`", T::EXPECT, t.found()), t)
        })
    }
}

// tokens

impl Parser {
    pub fn is_punct_of(&mut self, punct: char) -> Option<()> {
        self.peek().and_then(|e| match e {
            TokenTree::Punct(ok) if ok.as_char() == punct => Some(()),
            _ => None,
        })
    }

    pub fn is_punct_or_eof(&mut self, punct: char) -> bool {
        match self.peek() {
            Some(e) => matches!(e, TokenTree::Punct(p) if p.as_char() == punct),
            None => true,
        }
    }

    pub fn next_ident_of(&mut self, expect: &str) -> Option<Ident> {
        self.next_if_map(|e| match e {
            TokenTree::Ident(ok) if ok.as_str() == expect => Ok(ok),
            tree => Err(tree),
        })
    }

    pub fn next_group_of(&mut self, delim: Delimiter) -> Option<Group> {
        self.next_if_map(|e| match e {
            TokenTree::Group(ok) if ok.delimiter() == delim => Ok(ok),
            tree => Err(tree),
        })
    }

    pub fn next_punct_of(&mut self, punct: char) -> Option<Punct> {
        self.next_if_map(|e| match e {
            TokenTree::Punct(ok) if ok.as_char() == punct => Ok(ok),
            tree => Err(tree),
        })
    }

    pub fn ident_of(&mut self, expect: &str) -> Result<Ident> {
        self.try_next().and_then(|e|match e {
            TokenTree::Ident(ok) if ok.as_str() == expect => Ok(ok),
            t => Err(Error::new(format!("expected `{expect}`, found `{}`", t.found()), t)),
        })
    }

    pub fn group_of(&mut self, delim: Delimiter) -> Result<Group> {
        self.next_group_of(delim).ok_or_else(|| {
            Error::new_site(format!("expected `{}`, found `<eof>`", delim_punct(delim)))
        })
    }

    pub fn punct_of(&mut self, punct: char) -> Result<Punct> {
        self.next_punct_of(punct)
            .ok_or_else(|| Error::new_site(format!("expected `{punct}`, found `<eof>`")))
    }

    pub fn punctuated<T: Parse>(&mut self, sep: char) -> Result<Option<T>> {
        if self.is_empty() {
            return Ok(None);
        }
        let token = T::parse(self)?;
        let _ = self.next_punct_of(sep);
        Ok(Some(token))
    }
}
