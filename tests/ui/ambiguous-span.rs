use proc_macro2::Span;
use quote::quote_spanned;

trait CallSite {
    fn get() -> Self;
}

impl CallSite for Span {
    fn get() -> Self {
        Span::call_site()
    }
}

fn main() {
    let _ = quote_spanned!(CallSite::get()=> ...);
}
