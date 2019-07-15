use ext::TokenStreamExt;
pub use proc_macro2::*;
use ToTokens;

pub mod logic {
    use std::ops::BitOr;

    pub struct True;
    pub struct False;

    impl BitOr<False> for False {
        type Output = False;
        fn bitor(self, _rhs: False) -> False {
            False
        }
    }

    impl BitOr<False> for True {
        type Output = True;
        fn bitor(self, _rhs: False) -> True {
            True
        }
    }

    impl BitOr<True> for False {
        type Output = True;
        fn bitor(self, _rhs: True) -> True {
            True
        }
    }

    impl BitOr<True> for True {
        type Output = True;
        fn bitor(self, _rhs: True) -> True {
            True
        }
    }
}

pub trait HasIter {}
impl HasIter for logic::True {}
pub fn require_has_iter<T: HasIter>(_: T) {}

/// Extension traits used by the implementation of `quote!`. These are defined
/// in separate traits, rather than as a single trait due to ambiguity issues.
///
/// These traits expose a `__quote_into_iter` method which should allow calling
/// whichever impl happens to be applicable. Calling that method repeatedly on
/// the returned value should be idempotent.
pub mod ext {
    use std::slice;
    use ToTokens;

    use super::logic::{False as DoesNotHaveIter, True as HasIter};

    /// Extension trait providing the `__quote_into_iter` method on iterators.
    pub trait RepIteratorExt: Iterator + Sized {
        #[inline]
        fn __quote_into_iter(self) -> (Self, HasIter) {
            (self, HasIter)
        }
    }

    impl<T: Iterator> RepIteratorExt for T {}

    /// Extension trait providing the `__quote_into_iter` method for
    /// non-iterable types. These types don't produce an iterator and don't set
    /// the `_has_iter` outparameter.
    pub trait RepToTokensExt {
        /// Pretend to be an iterator for the purposes of `__quote_into_iter`.
        /// This allows repeated calls to `__quote_into_iter` to not set the
        /// `has_iter` outparameter.
        #[inline]
        fn next(&self) -> Option<&Self> {
            Some(self)
        }

        #[inline]
        fn __quote_into_iter<'a>(&'a self) -> (&'a Self, DoesNotHaveIter) {
            (self, DoesNotHaveIter)
        }
    }

    impl<T: ToTokens + ?Sized> RepToTokensExt for T {}

    /// Extension trait providing the `__quote_into_iter` method for types
    /// convertable into slices.
    ///
    /// NOTE: This is implemented manually, rather than by using a blanket impl
    /// over `AsRef<[T]>` to reduce the chance of ambiguity conflicts with other
    /// `__quote_into_iter` methods from this module.
    pub trait RepSliceExt {
        type Item;

        fn as_slice(&self) -> &[Self::Item];

        #[inline]
        fn __quote_into_iter<'a>(&'a self) -> (slice::Iter<'a, Self::Item>, HasIter) {
            (self.as_slice().iter(), HasIter)
        }
    }

    impl<'a, T: RepSliceExt + ?Sized> RepSliceExt for &'a T {
        type Item = T::Item;

        #[inline]
        fn as_slice(&self) -> &[Self::Item] {
            <T as RepSliceExt>::as_slice(*self)
        }
    }

    impl<'a, T: RepSliceExt + ?Sized> RepSliceExt for &'a mut T {
        type Item = T::Item;

        #[inline]
        fn as_slice(&self) -> &[Self::Item] {
            <T as RepSliceExt>::as_slice(*self)
        }
    }

    impl<T> RepSliceExt for [T] {
        type Item = T;

        #[inline]
        fn as_slice(&self) -> &[Self::Item] {
            self
        }
    }

    impl<T> RepSliceExt for Vec<T> {
        type Item = T;

        #[inline]
        fn as_slice(&self) -> &[Self::Item] {
            &self[..]
        }
    }

    macro_rules! array_rep_slice {
        ($($l:tt)*) => {
            $(
                impl<T> RepSliceExt for [T; $l] {
                    type Item = T;

                    #[inline]
                    fn as_slice(&self) -> &[Self::Item] {
                        &self[..]
                    }
                }
            )*
        }
    }

    array_rep_slice!(
        0 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16
        17 18 19 20 21 22 23 24 25 26 27 28 29 30 31 32
    );
}

// Helper type used within interpolations to allow for repeated binding names.
// Implements the relevant traits, and exports a dummy `next()` method.
#[derive(Copy, Clone)]
pub struct RepInterp<T>(pub T);

impl<T> RepInterp<T> {
    // This method is intended to look like `Iterator::next`, and is called when
    // a name is bound multiple times, as the previous binding will shadow the
    // original `Iterator` object. This allows us to avoid advancing the
    // iterator multiple times per iteration.
    #[inline]
    pub fn next(self) -> Option<T> {
        Some(self.0)
    }
}

impl<T: ext::RepSliceExt> ext::RepSliceExt for RepInterp<T> {
    type Item = T::Item;

    #[inline]
    fn as_slice(&self) -> &[Self::Item] {
        self.0.as_slice()
    }
}

impl<T: Iterator> Iterator for RepInterp<T> {
    type Item = T::Item;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

impl<T: ToTokens> ToTokens for RepInterp<T> {
    #[inline]
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.0.to_tokens(tokens);
    }
}

fn is_ident_start(c: u8) -> bool {
    (b'a' <= c && c <= b'z') || (b'A' <= c && c <= b'Z') || c == b'_'
}

fn is_ident_continue(c: u8) -> bool {
    (b'a' <= c && c <= b'z') || (b'A' <= c && c <= b'Z') || c == b'_' || (b'0' <= c && c <= b'9')
}

fn is_ident(token: &str) -> bool {
    let mut iter = token.bytes();
    let first_ok = iter.next().map(is_ident_start).unwrap_or(false);

    first_ok && iter.all(is_ident_continue)
}

pub fn parse(tokens: &mut TokenStream, span: Span, s: &str) {
    if is_ident(s) {
        // Fast path, since idents are the most common token.
        tokens.append(Ident::new(s, span));
    } else {
        let s: TokenStream = s.parse().expect("invalid token stream");
        tokens.extend(s.into_iter().map(|mut t| {
            t.set_span(span);
            t
        }));
    }
}

macro_rules! push_punct {
    ($name:ident $char1:tt) => {
        pub fn $name(tokens: &mut TokenStream, span: Span) {
            let mut punct = Punct::new($char1, Spacing::Alone);
            punct.set_span(span);
            tokens.append(punct);
        }
    };
    ($name:ident $char1:tt $char2:tt) => {
        pub fn $name(tokens: &mut TokenStream, span: Span) {
            let mut punct = Punct::new($char1, Spacing::Joint);
            punct.set_span(span);
            tokens.append(punct);
            let mut punct = Punct::new($char2, Spacing::Alone);
            punct.set_span(span);
            tokens.append(punct);
        }
    };
    ($name:ident $char1:tt $char2:tt $char3:tt) => {
        pub fn $name(tokens: &mut TokenStream, span: Span) {
            let mut punct = Punct::new($char1, Spacing::Joint);
            punct.set_span(span);
            tokens.append(punct);
            let mut punct = Punct::new($char2, Spacing::Joint);
            punct.set_span(span);
            tokens.append(punct);
            let mut punct = Punct::new($char3, Spacing::Alone);
            punct.set_span(span);
            tokens.append(punct);
        }
    };
}

push_punct!(push_add '+');
push_punct!(push_add_eq '+' '=');
push_punct!(push_and '&');
push_punct!(push_and_and '&' '&');
push_punct!(push_and_eq '&' '=');
push_punct!(push_at '@');
push_punct!(push_bang '!');
push_punct!(push_caret '^');
push_punct!(push_caret_eq '^' '=');
push_punct!(push_colon ':');
push_punct!(push_colon2 ':' ':');
push_punct!(push_comma ',');
push_punct!(push_div '/');
push_punct!(push_div_eq '/' '=');
push_punct!(push_dot '.');
push_punct!(push_dot2 '.' '.');
push_punct!(push_dot3 '.' '.' '.');
push_punct!(push_dot_dot_eq '.' '.' '=');
push_punct!(push_eq '=');
push_punct!(push_eq_eq '=' '=');
push_punct!(push_ge '>' '=');
push_punct!(push_gt '>');
push_punct!(push_le '<' '=');
push_punct!(push_lt '<');
push_punct!(push_mul_eq '*' '=');
push_punct!(push_ne '!' '=');
push_punct!(push_or '|');
push_punct!(push_or_eq '|' '=');
push_punct!(push_or_or '|' '|');
push_punct!(push_pound '#');
push_punct!(push_question '?');
push_punct!(push_rarrow '-' '>');
push_punct!(push_larrow '<' '-');
push_punct!(push_rem '%');
push_punct!(push_rem_eq '%' '=');
push_punct!(push_fat_arrow '=' '>');
push_punct!(push_semi ';');
push_punct!(push_shl '<' '<');
push_punct!(push_shl_eq '<' '<' '=');
push_punct!(push_shr '>' '>');
push_punct!(push_shr_eq '>' '>' '=');
push_punct!(push_star '*');
push_punct!(push_sub '-');
push_punct!(push_sub_eq '-' '=');
