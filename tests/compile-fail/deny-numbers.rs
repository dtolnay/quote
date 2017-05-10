#[macro_use]
extern crate quote;

fn main() {
    quote!(
        #{ 2*3*6 }
        //~^ ERROR expected ident, found 2
    );
}
