mod tokens;
pub use tokens::Tokens;

mod to_tokens;
pub use to_tokens::{ToTokens, ByteStr};

#[macro_export]
macro_rules! quote {
    ($($tt:tt)+) => {
        {
            #[allow(unused_imports)]
            use $crate::ToTokens;
            let mut _s = $crate::Tokens::new();
            quote_each_token!(_s $($tt)*);
            _s
        }
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

    ($tokens:ident # ( & $first:ident ) * $($rest:tt)*) => {
        $tokens.append_all(&$first);
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( & $first:ident $sep:tt ) * $($rest:tt)*) => {
        $tokens.append_terminated(&$first, stringify!($sep));
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( & $first:ident ) $sep:tt * $($rest:tt)*) => {
        $tokens.append_separated(&$first, stringify!($sep));
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $first:ident ) * $($rest:tt)*) => {
        $tokens.append_all($first);
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $first:ident $sep:tt ) * $($rest:tt)*) => {
        $tokens.append_terminated($first, stringify!($sep));
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $first:ident ) $sep:tt * $($rest:tt)*) => {
        $tokens.append_separated($first, stringify!($sep));
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # [ $($inner:tt)* ] $($rest:tt)*) => {
        $tokens.append("#");
        $tokens.append("[");
        quote_each_token!($tokens $($inner)*);
        $tokens.append("]");
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # $first:ident $($rest:tt)*) => {
        $first.to_tokens(&mut $tokens);
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
