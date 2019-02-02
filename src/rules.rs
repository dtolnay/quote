// Extract the names of all #metavariables and pass them to the $finish macro.
//
// in:   pounded_var_names!(then () a #b c #( #d )* #e)
// out:  then!(() b d e)
#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! pounded_var_names {
    ($finish:ident ($($found:ident)*) # ( $($inner:tt)* ) $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) # [ $($inner:tt)* ] $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
    };

    ($finish:ident ($($found:ident)*) # { $($inner:tt)* } $($rest:tt)*) => {
        pounded_var_names!($finish ($($found)*) $($inner)* $($rest)*)
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
#[macro_export(local_inner_macros)]
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
#[macro_export(local_inner_macros)]
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

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! quote_each_token {
    ($tokens:ident $span:ident) => {};

    ($tokens:ident $span:ident # ! $($rest:tt)*) => {
        quote_each_token!($tokens $span #);
        quote_each_token!($tokens $span !);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident # ( $($inner:tt)* ) * $($rest:tt)*) => {
        for pounded_var_names!(nested_tuples_pat () $($inner)*)
        in pounded_var_names!(multi_zip_expr () $($inner)*) {
            quote_each_token!($tokens $span $($inner)*);
        }
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident # ( $($inner:tt)* ) $sep:tt * $($rest:tt)*) => {
        for (_i, pounded_var_names!(nested_tuples_pat () $($inner)*))
        in pounded_var_names!(multi_zip_expr () $($inner)*).into_iter().enumerate() {
            if _i > 0 {
                quote_each_token!($tokens $span $sep);
            }
            quote_each_token!($tokens $span $($inner)*);
        }
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident # [ $($inner:tt)* ] $($rest:tt)*) => {
        quote_each_token!($tokens $span #);
        $tokens.extend({
            let mut g = $crate::__rt::Group::new(
                $crate::__rt::Delimiter::Bracket,
                quote_spanned!($span=> $($inner)*),
            );
            g.set_span($span);
            Some($crate::__rt::TokenTree::from(g))
        });
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident # $first:ident $($rest:tt)*) => {
        $crate::ToTokens::to_tokens(&$first, &mut $tokens);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ( $($first:tt)* ) $($rest:tt)*) => {
        $tokens.extend({
            let mut g = $crate::__rt::Group::new(
                $crate::__rt::Delimiter::Parenthesis,
                quote_spanned!($span=> $($first)*),
            );
            g.set_span($span);
            Some($crate::__rt::TokenTree::from(g))
        });
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident [ $($first:tt)* ] $($rest:tt)*) => {
        $tokens.extend({
            let mut g = $crate::__rt::Group::new(
                $crate::__rt::Delimiter::Bracket,
                quote_spanned!($span=> $($first)*),
            );
            g.set_span($span);
            Some($crate::__rt::TokenTree::from(g))
        });
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident { $($first:tt)* } $($rest:tt)*) => {
        $tokens.extend({
            let mut g = $crate::__rt::Group::new(
                $crate::__rt::Delimiter::Brace,
                quote_spanned!($span=> $($first)*),
            );
            g.set_span($span);
            Some($crate::__rt::TokenTree::from(g))
        });
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident + $($rest:tt)*) => {
        $crate::__rt::push_add(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident += $($rest:tt)*) => {
        $crate::__rt::push_add_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident & $($rest:tt)*) => {
        $crate::__rt::push_and(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident && $($rest:tt)*) => {
        $crate::__rt::push_and_and(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident &= $($rest:tt)*) => {
        $crate::__rt::push_and_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident @ $($rest:tt)*) => {
        $crate::__rt::push_at(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ! $($rest:tt)*) => {
        $crate::__rt::push_bang(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ^ $($rest:tt)*) => {
        $crate::__rt::push_caret(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ^= $($rest:tt)*) => {
        $crate::__rt::push_caret_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident : $($rest:tt)*) => {
        $crate::__rt::push_colon(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident :: $($rest:tt)*) => {
        $crate::__rt::push_colon2(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident , $($rest:tt)*) => {
        $crate::__rt::push_comma(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident / $($rest:tt)*) => {
        $crate::__rt::push_div(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident /= $($rest:tt)*) => {
        $crate::__rt::push_div_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident . $($rest:tt)*) => {
        $crate::__rt::push_dot(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident .. $($rest:tt)*) => {
        $crate::__rt::push_dot2(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ... $($rest:tt)*) => {
        $crate::__rt::push_dot3(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ..= $($rest:tt)*) => {
        $crate::__rt::push_dot_dot_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident = $($rest:tt)*) => {
        $crate::__rt::push_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident == $($rest:tt)*) => {
        $crate::__rt::push_eq_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident >= $($rest:tt)*) => {
        $crate::__rt::push_ge(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident > $($rest:tt)*) => {
        $crate::__rt::push_gt(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident <= $($rest:tt)*) => {
        $crate::__rt::push_le(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident < $($rest:tt)*) => {
        $crate::__rt::push_lt(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident *= $($rest:tt)*) => {
        $crate::__rt::push_mul_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident != $($rest:tt)*) => {
        $crate::__rt::push_ne(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident | $($rest:tt)*) => {
        $crate::__rt::push_or(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident |= $($rest:tt)*) => {
        $crate::__rt::push_or_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident || $($rest:tt)*) => {
        $crate::__rt::push_or_or(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident # $($rest:tt)*) => {
        $crate::__rt::push_pound(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ? $($rest:tt)*) => {
        $crate::__rt::push_question(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident -> $($rest:tt)*) => {
        $crate::__rt::push_rarrow(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident <- $($rest:tt)*) => {
        $crate::__rt::push_larrow(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident % $($rest:tt)*) => {
        $crate::__rt::push_rem(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident %= $($rest:tt)*) => {
        $crate::__rt::push_rem_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident => $($rest:tt)*) => {
        $crate::__rt::push_fat_arrow(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident ; $($rest:tt)*) => {
        $crate::__rt::push_semi(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident << $($rest:tt)*) => {
        $crate::__rt::push_shl(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident <<= $($rest:tt)*) => {
        $crate::__rt::push_shl_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident >> $($rest:tt)*) => {
        $crate::__rt::push_shr(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident >>= $($rest:tt)*) => {
        $crate::__rt::push_shr_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident * $($rest:tt)*) => {
        $crate::__rt::push_star(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident - $($rest:tt)*) => {
        $crate::__rt::push_sub(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident -= $($rest:tt)*) => {
        $crate::__rt::push_sub_eq(&mut $tokens, $span);
        quote_each_token!($tokens $span $($rest)*);
    };

    ($tokens:ident $span:ident $first:tt $($rest:tt)*) => {
        $crate::__rt::parse(&mut $tokens, $span, quote_stringify!($first));
        quote_each_token!($tokens $span $($rest)*);
    };
}

// Unhygienically invoke whatever `stringify` the caller has in scope i.e. not a
// local macro. The macros marked `local_inner_macros` above cannot invoke
// `stringify` directly.
#[macro_export]
#[doc(hidden)]
macro_rules! quote_stringify {
    ($tt:tt) => {
        stringify!($tt)
    };
}
