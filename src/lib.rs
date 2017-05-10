//! Quasi-quoting without a Syntex dependency, intended for use with [Macros
//! 1.1](https://github.com/rust-lang/rfcs/blob/master/text/1681-macros-1.1.md).
//!
//! ```toml
//! [dependencies]
//! quote = "0.3"
//! ```
//!
//! ```rust,ignore
//! #[macro_use]
//! extern crate quote;
//! ```
//!
//! Interpolation is done with `#var`:
//!
//! ```text
//! let tokens = quote! {
//!     struct SerializeWith #generics #where_clause {
//!         value: &'a #field_ty,
//!         phantom: ::std::marker::PhantomData<#item_ty>,
//!     }
//!
//!     impl #generics serde::Serialize for SerializeWith #generics #where_clause {
//!         fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
//!             where S: serde::Serializer
//!         {
//!             #path(self.value, s)
//!         }
//!     }
//!
//!     SerializeWith {
//!         value: #value,
//!         phantom: ::std::marker::PhantomData::<#item_ty>,
//!     }
//! };
//! ```
//!
//! Repetition is done using `#(...)*` or `#(...),*` very similar to `macro_rules!`:
//!
//! - `#(#var)*` - no separators
//! - `#(#var),*` - the character before the asterisk is used as a separator
//! - `#( struct #var; )*` - the repetition can contain other things
//! - `#( #k => println!("{}", #v), )*` - even multiple interpolations
//!
//! The return type of `quote!` is `quote::Tokens`. Tokens can be interpolated into
//! other quotes:
//!
//! ```text
//! let t = quote! { /* ... */ };
//! return quote! { /* ... */ #t /* ... */ };
//! ```
//!
//! You can use `#{...}` for some basic computations inside of quotations. Computations are
//! restricted to combinations of:
//!
//! - `#{x[0]}` - index arrays with integers 0..32
//! - `#{x.0}` - index tuple structs with integers 0..32
//! - `#{x.foo}` - access fields
//! - `#{x()}` - call functions (without arguments)
//!
//! Note:
//! - Any chained combination of the above is also possible, like `#{self.foo[0].bar()}`.
//!   But please consider replacing too complex computations with helper variables `#bar`
//!   in order to improve the readability for other people - thank you in advance. ;-)
//! - These computations can be particularly useful if you want to implement `quote::ToTokens`
//!   for a custom struct. You can then reference the struct fields directly via `#{self.foo}`
//!   etc.
//! - computations `#{...}` _inside_ of repetitions `#(...)*` are treated as constant expressions
//!   and are not iterated over - in contrast to any other `#x` inside of a repetition.
//!
//! Call `to_string()` or `as_str()` on a Tokens to get a `String` or `&str` of Rust
//! code.
//!
//! The `quote!` macro relies on deep recursion so some large invocations may fail
//! with "recursion limit reached" when you compile. If it fails, bump up the
//! recursion limit by adding `#![recursion_limit = "128"]` to your crate. An even
//! higher limit may be necessary for especially large invocations.

mod tokens;
pub use tokens::Tokens;

mod to_tokens;
pub use to_tokens::{ToTokens, ByteStr, Hex};

mod ident;
pub use ident::Ident;

/// The whole point.
#[macro_export]
macro_rules! quote {
    () => {
        $crate::Tokens::new()
    };

    ($($tt:tt)+) => {
        {
            let mut _s = $crate::Tokens::new();
            quote_each_token!(_s $($tt)*);
            _s
        }
    };
}

// Extract the names of all #metavariables and pass them to the $finish macro.
//
// in:   pounded_var_names!(then () a #b c #( #d )* #e)
// out:  then!(() b d e)
#[macro_export]
#[doc(hidden)]
macro_rules! pounded_var_names {
    ($finish:ident ($($found:ident)*) # ( $($inner:tt)* ) $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) # [ $($inner:tt)* ] $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) # { $($ignore:tt)* } $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($rest)*)
    };

    ($finish:ident ($($found:ident)*) # $first:ident $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)* $first) $($rest)*)
    };

    ($finish:ident ($($found:ident)*) ( $($inner:tt)* ) $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) [ $($inner:tt)* ] $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) { $($inner:tt)* } $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) $ignore:tt $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($rest)*)
    };

    ($finish:ident ($($found:ident)*)) => {
        $finish!(() $($found)*)
    };
}

// in:   nested_tuples_pat!(() a b c d e)
// out:  ((((a b) c) d) e)
//
// in:   nested_tuples_pat!(() a)
// out:  a
#[macro_export]
#[doc(hidden)]
macro_rules! nested_tuples_pat {
    (()) => {
        &()
    };

    (() $first:ident $($rest:ident)*) => {
        nested_tuples_pat!(($first) $($rest)*)
    };

    (($pat:pat) $first:ident $($rest:ident)*) => {
        nested_tuples_pat!((($pat, $first)) $($rest)*)
    };

    (($done:pat)) => {
        $done
    };
}

// in:   multi_zip_expr!(() a b c d e)
// out:  a.into_iter().zip(b).zip(c).zip(d).zip(e)
//
// in:   multi_zip_iter!(() a)
// out:  a
#[macro_export]
#[doc(hidden)]
macro_rules! multi_zip_expr {
    (()) => {
        &[]
    };

    (() $single:ident) => {
        $single
    };

    (() $first:ident $($rest:ident)*) => {
        multi_zip_expr!(($first.into_iter()) $($rest)*)
    };

    (($zips:expr) $first:ident $($rest:ident)*) => {
        multi_zip_expr!(($zips.zip($first)) $($rest)*)
    };

    (($done:expr)) => {
        $done
    };
}

// in: validate_computation_in_interpolation!(@COMPUTATION self.foo())
// result: OK - empty token tree
//
// in: validate_computation_in_interpolation!(@COMPUTATION self.x.map(|x| 3*x))
// result: Parsing error "unexpected Token `|`"
#[macro_export]
#[doc(hidden)]
macro_rules! validate_computation_in_interpolation {
    // input must either start with `($ident...)`
    //
    // -> in order to support the computation `#{(matrix.2).3}`
    (@ENTRY ($ident:ident $($branch:tt)*) $($rest:tt)*) => {
        validate_computation_in_interpolation!(@AFTER_IDENT $($branch)*);
        validate_computation_in_interpolation!(@AFTER_IDENT $($rest)*);
    };

    // .. or it must start with an identifier `$ident`
    (@ENTRY $ident:ident $($rest:tt)*) => {
        validate_computation_in_interpolation!(@AFTER_IDENT $($rest)*);
    };

    // done when empty
    (@AFTER_IDENT) => {};

    // function calls are ok - but only without arguments
    (@AFTER_IDENT () $($rest:tt)*) => {
        validate_computation_in_interpolation!(@AFTER_IDENT $($rest)*);
    };

    // struct access is ok - both by field ident or tuple index
    (@AFTER_IDENT . $expect_number_or_ident:tt $($rest:tt)*) => {
        validate_computation_in_interpolation!(@IS [NUMBER IDENT] $expect_number_or_ident);
        validate_computation_in_interpolation!(@AFTER_IDENT $($rest)*);
    };

    // array access is ok - but only with for indices 0..32
    (@AFTER_IDENT [$($expect_number:tt)*] $($rest:tt)*) => {
        validate_computation_in_interpolation!(@IS [NUMBER] $($expect_number)*);
        validate_computation_in_interpolation!(@AFTER_IDENT $($rest)*);
    };

    // or else produce a nicer compiler error: "no rules expected the token `$invalid`"
    (@AFTER_IDENT $invalid:tt $($rest:tt)*) => {
        // no rule accepts a single token - voila!
        validate_computation_in_interpolation!($invalid);
    };

    // -----------------------------------------
    // `@IS [NUMBER IDENT] something` will check whether `something` is either a number or an
    // identifier
    //
    // Important: `@IS [NUMBER IDENT]` will work, but `@IS [IDENT NUMBER]` will fail, see
    // - https://github.com/rust-lang/rust/issues/27832
    //
    // The following will parse
    // - @IS [NUMBER] 0
    // - @IS [IDENT] foo
    // - @IS [NUMBER IDENT] 0
    //
    // The following will not parse
    // - @IS [NUMBER] foo
    // - @IS [IDENT] 0
    // -----------------------------------------
    // if ident
    (@IS [IDENT $($or:tt)*] $ident:ident) => {};
    // .. else continue
    (@IS [IDENT $($or:tt)*] $($rest:tt)*) => {
        validate_computation_in_interpolation!(@IS [$($or)*] $($rest)*);
    };
    // if number in 0..32
    (@IS [NUMBER $($or:tt)*] 0) => {};
    (@IS [NUMBER $($or:tt)*] 1) => {};
    (@IS [NUMBER $($or:tt)*] 2) => {};
    (@IS [NUMBER $($or:tt)*] 3) => {};
    (@IS [NUMBER $($or:tt)*] 4) => {};
    (@IS [NUMBER $($or:tt)*] 5) => {};
    (@IS [NUMBER $($or:tt)*] 6) => {};
    (@IS [NUMBER $($or:tt)*] 7) => {};
    (@IS [NUMBER $($or:tt)*] 8) => {};
    (@IS [NUMBER $($or:tt)*] 9) => {};
    (@IS [NUMBER $($or:tt)*] 10) => {};
    (@IS [NUMBER $($or:tt)*] 11) => {};
    (@IS [NUMBER $($or:tt)*] 12) => {};
    (@IS [NUMBER $($or:tt)*] 13) => {};
    (@IS [NUMBER $($or:tt)*] 14) => {};
    (@IS [NUMBER $($or:tt)*] 15) => {};
    (@IS [NUMBER $($or:tt)*] 16) => {};
    (@IS [NUMBER $($or:tt)*] 17) => {};
    (@IS [NUMBER $($or:tt)*] 18) => {};
    (@IS [NUMBER $($or:tt)*] 19) => {};
    (@IS [NUMBER $($or:tt)*] 20) => {};
    (@IS [NUMBER $($or:tt)*] 21) => {};
    (@IS [NUMBER $($or:tt)*] 22) => {};
    (@IS [NUMBER $($or:tt)*] 23) => {};
    (@IS [NUMBER $($or:tt)*] 24) => {};
    (@IS [NUMBER $($or:tt)*] 25) => {};
    (@IS [NUMBER $($or:tt)*] 26) => {};
    (@IS [NUMBER $($or:tt)*] 27) => {};
    (@IS [NUMBER $($or:tt)*] 28) => {};
    (@IS [NUMBER $($or:tt)*] 29) => {};
    (@IS [NUMBER $($or:tt)*] 30) => {};
    (@IS [NUMBER $($or:tt)*] 31) => {};
    // .. else continue
    (@IS [NUMBER $($or:tt)*] $($rest:tt)*) => {
        validate_computation_in_interpolation!(@IS [$($or)*] $($rest)*);
    };
    // if we reached `@IS []`, then nothing will match.
    //
    // Let's apply a trick to produce a nicer compiler error
    (@IS [] $invalid:tt $($rest:tt)*) => {
        // no rule accepts a single token - voila!
        validate_computation_in_interpolation!($invalid);
    };
}

#[macro_export]
#[doc(hidden)]
macro_rules! quote_each_token {
    ($tokens:ident) => {};

    ($tokens:ident # ! $($rest:tt)*) => {
        $tokens.append("#");
        $tokens.append("!");
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $($inner:tt)* ) * $($rest:tt)*) => {
        for pounded_var_names!(nested_tuples_pat () $($inner)*)
        in pounded_var_names!(multi_zip_expr () $($inner)*) {
            quote_each_token!($tokens $($inner)*);
        }
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $($inner:tt)* ) $sep:tt * $($rest:tt)*) => {
        for (_i, pounded_var_names!(nested_tuples_pat () $($inner)*))
        in pounded_var_names!(multi_zip_expr () $($inner)*).into_iter().enumerate() {
            if _i > 0 {
                $tokens.append(stringify!($sep));
            }
            quote_each_token!($tokens $($inner)*);
        }
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # [ $($inner:tt)* ] $($rest:tt)*) => {
        $tokens.append("#");
        $tokens.append("[");
        quote_each_token!($tokens $($inner)*);
        $tokens.append("]");
        quote_each_token!($tokens $($rest)*);
    };

    // wrap computations in a
    ($tokens:ident # { $($inner:tt)* } $($rest:tt)*) => {
        validate_computation_in_interpolation!(@ENTRY $($inner)*);
        let computation = &$($inner)*;
        $crate::ToTokens::to_tokens(computation, &mut $tokens);
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # $first:ident $($rest:tt)*) => {
        $crate::ToTokens::to_tokens(&$first, &mut $tokens);
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident ( $($first:tt)* ) $($rest:tt)*) => {
        $tokens.append("(");
        quote_each_token!($tokens $($first)*);
        $tokens.append(")");
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident [ $($first:tt)* ] $($rest:tt)*) => {
        $tokens.append("[");
        quote_each_token!($tokens $($first)*);
        $tokens.append("]");
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident { $($first:tt)* } $($rest:tt)*) => {
        $tokens.append("{");
        quote_each_token!($tokens $($first)*);
        $tokens.append("}");
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident $first:tt $($rest:tt)*) => {
        $tokens.append(stringify!($first));
        quote_each_token!($tokens $($rest)*);
    };
}
