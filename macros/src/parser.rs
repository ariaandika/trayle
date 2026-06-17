use crate::tree::*;

use Delimiter as Delim;
use TokenTree as Tree;

use crate::Error;

macro_rules! errfmt {
    ($me:ident, $($tt:tt)*) => {
        Error::new(format!($($tt)*), $me.span())
    };
}

// ===== trait =====

pub trait Parse: Sized {
    fn parse(parser: &mut Parser) -> Result<Self, Error>;
}

// ===== Parser =====

pub struct Parser {
    iter: IntoIter,
    cache: [Option<Tree>; 2],
}

impl Parser {
    pub fn new(tokens: TokenStream) -> Self {
        Self {
            iter: tokens.into_iter(),
            cache: [const { None }; _],
        }
    }

    pub fn next(&mut self) -> Option<Tree> {
        match self.cache[0].take() {
            Some(cached) => {
                self.cache[0] = self.cache[1].take();
                Some(cached)
            }
            None => self.iter.next(),
        }
    }

    pub fn next_if_map<O, F: FnOnce(Tree) -> Result<O, Tree>>(
        &mut self,
        f: F,
    ) -> Option<O> {
        let tree = self.cache[0].take().or_else(|| self.next())?;
        match f(tree) {
            Ok(o) => {
                self.cache[0] = self.cache[1].take();
                Some(o)
            }
            Err(tree) => {
                self.cache[0] = Some(tree);
                None
            }
        }
    }

    pub fn peek(&mut self) -> Option<&Tree> {
        if self.cache[0].is_none() {
            self.cache[0] = self.iter.next();
        }
        self.cache[0].as_ref()
    }

    pub fn span(&mut self) -> Span {
        self.peek().map(Tree::span).unwrap_or_else(Span::call_site)
    }

    pub fn parse<T: Parse>(&mut self) -> Result<T, Error> {
        T::parse(self)
    }
}

// TokenTree

impl Parser {
    next_tree!(next_ident(self) -> Ident);
    try_tree!(ident(self) -> Ident, "identifier");
    try_tree!(lit(self) -> Literal, "literal");

    pub fn is_punct_of(&mut self, punct: char) -> Option<()> {
        self.peek().and_then(|e| match e {
            Tree::Punct(ok) if ok.as_char() == punct => Some(()),
            _ => None,
        })
    }

    pub fn next_ident_of(&mut self, expect: &str) -> Option<Ident> {
        self.next_if_map(|e| match e {
            TokenTree::Ident(ok) if ok.to_string() == expect => Ok(ok),
            tree => Err(tree),
        })
    }

    pub fn next_group_of(&mut self, delim: Delim) -> Option<Group> {
        self.next_if_map(|e| match e {
            Tree::Group(ok) if ok.delimiter() == delim => Ok(ok),
            tree => Err(tree),
        })
    }

    pub fn next_punct_of(&mut self, punct: char) -> Option<Punct> {
        self.next_if_map(|e| match e {
            Tree::Punct(ok) if ok.as_char() == punct => Ok(ok),
            tree => Err(tree),
        })
    }

    pub fn ident_of(&mut self, expect: &str) -> Result<Ident, Error> {
        self.next_ident_of(expect).ok_or_else(Error::eof)
    }

    pub fn group_of(&mut self, delim: Delim) -> Result<Group, Error> {
        self.next_group_of(delim)
            .ok_or_else(|| errfmt!(self, "expected `{}`", delim_punct(delim)))
    }

    pub fn punct_of(&mut self, punct: char) -> Result<Punct, Error> {
        self.next_punct_of(punct)
            .ok_or_else(|| errfmt!(self, "expected `{punct}`"))
    }
}

fn delim_punct(delim: Delim) -> &'static str {
    match delim {
        Delim::Parenthesis => "(",
        Delim::Brace => "{",
        Delim::Bracket => "[",
        Delim::None => "no delimiter"
    }
}

macro_rules! try_tree {
    ($fn:ident($me:ident) -> $tr:ident, $ex:literal) => {
        pub fn $fn(&mut $me) -> Result<$tr, Error> {
            match $me.next() {
                Some(Tree::$tr(ok)) => Ok(ok),
                Some(span) => Err(Error::new(
                    format!(concat!("expected ", $ex, ", found {:?}"), span),
                    span.span()
                )),
                None => Err(Error::new("unexpected EOF".into(), Span::call_site())),
            }
        }
    };
}


macro_rules! next_tree {
    ($fn:ident($me:ident) -> $tr:ident) => {
        pub fn $fn(&mut $me) -> Option<$tr> {
            $me.next_if_map(|e| match e {
                Tree::$tr(ok) => Ok(ok),
                tree => Err(tree),
            })
        }
    };
}

use {try_tree, next_tree};
