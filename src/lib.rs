mod tokens;
pub use tokens::Tokens;

mod to_tokens;
pub use to_tokens::ToTokens;

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
        $tokens.append("#!");
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $first:ident ) * $($rest:tt)*) => {
        for _v in &$first {
            _v.to_tokens(&mut $tokens);
        }
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $first:ident $sep:tt ) * $($rest:tt)*) => {
        for _v in &$first {
            _v.to_tokens(&mut $tokens);
            $tokens.append(stringify!($sep));
        }
        quote_each_token!($tokens $($rest)*);
    };

    ($tokens:ident # ( $first:ident ) $sep:tt * $($rest:tt)*) => {
        for (_i, _v) in $first.iter().enumerate() {
            if _i > 0 {
                $tokens.append(stringify!($sep));
            }
            _v.to_tokens(&mut $tokens);
        }
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
