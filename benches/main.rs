#![allow(unknown_lints, special_module_name)]
#![allow(
    clippy::cast_lossless,
    clippy::let_underscore_untyped,
    clippy::uninlined_format_args
)]

quote_benchmark::run_quote_benchmark!(_);

mod benchmark {
    macro_rules! benchmark {
        (|$ident:ident| $quote:expr) => {
            use proc_macro2::{Ident, Span};

            pub fn quote() -> proc_macro2::TokenStream {
                let $ident = Ident::new("Response", Span::call_site());
                $quote
            }
        };
    }

    pub(crate) use benchmark;
}

use benchmark::benchmark;

mod lib;
mod timer;

fn main() {
    timer::time("non-macro", lib::quote);
}
