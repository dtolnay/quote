use super::ToTokens;
use std::fmt::{self, Debug, Display};

use proc_macro;
use proc_macro2::{TokenStream, TokenTree};

/// Tokens produced by a `quote!(...)` invocation.
#[derive(Clone, Default)]
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
    where
        U: Into<TokenTree>,
    {
        self.tts.push(token.into());
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
    where
        T: ToTokens,
        I: IntoIterator<Item = T>,
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
    where
        T: ToTokens,
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
    where
        T: ToTokens,
        I: IntoIterator<Item = T>,
        U: ToTokens,
    {
        for token in iter {
            token.to_tokens(self);
            term.to_tokens(self);
        }
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

impl From<Tokens> for TokenStream {
    fn from(tokens: Tokens) -> TokenStream {
        tokens.tts.into_iter().collect()
    }
}

impl From<Tokens> for proc_macro::TokenStream {
    fn from(tokens: Tokens) -> proc_macro::TokenStream {
        TokenStream::from(tokens).into()
    }
}

impl Display for Tokens {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        Display::fmt(&TokenStream::from(self.clone()), formatter)
    }
}

impl Debug for Tokens {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        struct DebugAsDisplay<'a, T: 'a>(&'a T);

        impl<'a, T> Debug for DebugAsDisplay<'a, T>
        where
            T: Display,
        {
            fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                Display::fmt(self.0, formatter)
            }
        }

        formatter
            .debug_tuple("Tokens")
            .field(&DebugAsDisplay(self))
            .finish()
    }
}
