#[macro_use]
extern crate quote;

fn main() {
    quote!(
        #{ a+b }
        //~^ ERROR no rules expected the token `+`
    );
}
