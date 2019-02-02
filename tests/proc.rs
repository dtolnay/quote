extern crate proc_macro2;
extern crate quote;

use proc_macro2::TokenStream;
use quote::quote;

#[test]
fn test() {
    let x = 0i32;
    let tts: TokenStream = quote!(x ::= "x" [#x q r#struct]);
    assert_eq!("x :: = \"x\" [ 0i32 q r#struct ]", tts.to_string());
}
