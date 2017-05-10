#[macro_use]
extern crate quote;

fn main() {
    quote!(
        #{ format!("Hello {}!", "world") }
        //~^ ERROR no rules expected the token `!`
    );
}
