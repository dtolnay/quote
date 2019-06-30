use ext::TokenStreamExt;
use ToTokens;
pub use proc_macro2::*;

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

impl<T: IntoIterator> IntoIterator for RepInterp<T> {
    type Item = T::Item;
    type IntoIter = T::IntoIter;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
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
