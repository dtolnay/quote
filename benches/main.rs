quote_benchmark::run_quote_benchmark!();

mod benchmark {
    macro_rules! benchmark {
        ($quote:expr) => {
            pub fn quote() -> proc_macro2::TokenStream {
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
