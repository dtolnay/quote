#[macro_use]
extern crate quote;

fn main() {
    quote!(
        #{ foo.map(|x| &x) }
        //~^ ERROR no rules expected the token `(`
    );
}
