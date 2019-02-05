#[macro_export]
#[doc(hidden)]
macro_rules! quote_each_token_default_span {
    ($($tt:tt)*) => {{
        #[derive($crate::QuoteImpl)]
        enum QuoteHack {
            Value = (stringify! { $($tt)* }, 0).1,
        }
        proc_macro_call!()
    }};
}

#[macro_export]
#[doc(hidden)]
macro_rules! quote_each_token {
    ($($tt:tt)*) => {{
        #[derive($crate::QuoteSpannedImpl)]
        enum QuoteHack {
            Value = (stringify! { $($tt)* }, 0).1,
        }
        proc_macro_call!()
    }};
}
