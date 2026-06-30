use std::iter::Flatten;
use std::option::IntoIter as OptIter;

use crate::tree::*;

// ===== Either =====

#[derive(Debug, Clone)]
pub enum Either<L, R> {
    Left(L),
    Right(R),
}

impl<L: Iterator, R: Iterator<Item = L::Item>> Iterator for Either<L, R> {
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        match self {
            Either::Left(l) => l.size_hint(),
            Either::Right(r) => r.size_hint(),
        }
    }
}

// ===== bool ext =====

pub trait BoolExt: Sized {
    fn then_stream<I, F>(self, f: F) -> Flatten<OptIter<I::IntoIter>>
    where
        I: IntoIterator<Item = TokenTree>,
        F: FnOnce() -> I;
}

impl BoolExt for bool {
    fn then_stream<I, F>(self, f: F) -> Flatten<OptIter<I::IntoIter>>
    where
        I: IntoIterator<Item = TokenTree>,
        F: FnOnce() -> I,
    {
        self.then(|| f().into_iter()).into_iter().flatten()
    }
}

// ===== option ext =====

pub trait OptionExt<T> {
    fn map_stream<I, F>(self, f: F) -> Flatten<OptIter<I>>
    where
        I: Iterator<Item = TokenTree>,
        F: FnOnce(T) -> I;
}

impl<T> OptionExt<T> for Option<T> {
    fn map_stream<I, F>(self, f: F) -> Flatten<OptIter<I>>
    where
        I: Iterator<Item = TokenTree>,
        F: FnOnce(T) -> I,
    {
        self.map(f).into_iter().flatten()
    }
}

// ===== iterator ext =====

pub trait IteratorExt: Sized + Iterator<Item = TokenTree> {
    fn map_group(self, delim: Delimiter) -> OptIter<TokenTree> {
        Some(Group::new(delim, self.collect()).into()).into_iter()
    }

    fn chain_back(self, other: impl Iterator<Item = TokenTree>) -> impl Iterator<Item = TokenTree> {
        other.chain(self)
    }
}

impl<I: Iterator<Item = TokenTree>> IteratorExt for I {}

