use super::ToTokens;
use std::fmt::{self, Display};

use proc_macro2::{TokenStream, TokenTree, TokenNode, Term, Span};
use proc_macro2::Delimiter;

/// Tokens produced by a `quote!(...)` invocation.
#[derive(Clone)]
pub struct Tokens {
    tts: Vec<TokenTree>,
}

impl Tokens {
    /// Empty tokens.
    pub fn new() -> Self {
        Tokens { tts: Vec::new() }
    }

    /// For use by `ToTokens` implementations.
    ///
    /// Appends the token specified to this list of tokens.
    pub fn append<U>(&mut self, token: U)
        where U: Into<TokenTree>,
    {
        self.tts.push(token.into());
    }

    /// Add `tokens` into `self`.
    pub fn append_tokens<T: ToTokens>(&mut self, tokens: T) {
        tokens.to_tokens(self)
    }

    /// Add the symbol specified to this list of tokens.
    pub fn append_sym(&mut self, sym: &str, span: Span) {
        self.append(TokenTree {
            span: span,
            kind: TokenNode::Term(Term::intern(sym)),
        });
    }

    pub fn append_delimited<F, R>(&mut self,
                                  delim: &str,
                                  span: Span,
                                  f: F) -> R
        where F: FnOnce(&mut Tokens) -> R,
    {
        let delim = match delim {
            "(" => Delimiter::Parenthesis,
            "[" => Delimiter::Bracket,
            "{" => Delimiter::Brace,
            _ => panic!("unknown delimiter: {}", delim),
        };
        let mut child = Tokens::new();
        let ret = f(&mut child);
        self.append(TokenTree {
            span: span,
            kind: TokenNode::Group(delim, child.into()),
        });
        return ret
    }

    /// For use by `ToTokens` implementations.
    ///
    /// ```
    /// # #[macro_use] extern crate quote;
    /// # use quote::{Tokens, ToTokens};
    /// # fn main() {
    /// struct X;
    ///
    /// impl ToTokens for X {
    ///     fn to_tokens(&self, tokens: &mut Tokens) {
    ///         tokens.append_all(&[true, false]);
    ///     }
    /// }
    ///
    /// let tokens = quote!(#X);
    /// assert_eq!(tokens.to_string(), "true false");
    /// # }
    /// ```
    pub fn append_all<T, I>(&mut self, iter: I)
        where T: ToTokens,
              I: IntoIterator<Item = T>
    {
        for token in iter {
            token.to_tokens(self);
        }
    }

    /// For use by `ToTokens` implementations.
    ///
    /// Appends all of the items in the iterator `I`, separated by the tokens
    /// `U`.
    pub fn append_separated<T, I, U>(&mut self, iter: I, op: U)
        where T: ToTokens,
              I: IntoIterator<Item = T>,
              U: ToTokens,
    {
        for (i, token) in iter.into_iter().enumerate() {
            if i > 0 {
                op.to_tokens(self);
            }
            token.to_tokens(self);
        }
    }

    /// For use by `ToTokens` implementations.
    ///
    /// Appends all tokens in the iterator `I`, appending `U` after each
    /// element, including after the last element of the iterator.
    pub fn append_terminated<T, I, U>(&mut self, iter: I, term: U)
        where T: ToTokens,
              I: IntoIterator<Item = T>,
              U: ToTokens,
    {
        for token in iter {
            token.to_tokens(self);
            term.to_tokens(self);
        }
    }
}

impl From<Tokens> for TokenStream {
    fn from(tokens: Tokens) -> TokenStream {
        tokens.tts.into_iter().collect()
    }
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens::new()
    }
}

impl ToTokens for Tokens {
    fn to_tokens(&self, dst: &mut Tokens) {
        dst.tts.extend(self.tts.iter().cloned());
    }

    fn into_tokens(self) -> Tokens {
        self
    }
}

impl ToTokens for TokenStream {
    fn to_tokens(&self, dst: &mut Tokens) {
        dst.append_all(self.clone().into_iter());
    }
}

impl ToTokens for TokenTree {
    fn to_tokens(&self, dst: &mut Tokens) {
        dst.append(self.clone());
    }
}

impl Display for Tokens {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        TokenStream::from(self.clone()).fmt(formatter)
    }
}
