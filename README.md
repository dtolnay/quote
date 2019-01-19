Rust Quasi-Quoting
==================

[![Build Status](https://api.travis-ci.org/dtolnay/quote.svg?branch=master)](https://travis-ci.org/dtolnay/quote)
[![Latest Version](https://img.shields.io/crates/v/quote.svg)](https://crates.io/crates/quote)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/quote/)

This crate provides the [`quote!`] macro for turning Rust syntax tree data
structures into tokens of source code.

[`quote!`]: https://docs.rs/quote/0.6/quote/macro.quote.html

Procedural macros in Rust receive a stream of tokens as input, execute arbitrary
Rust code to determine how to manipulate those tokens, and produce a stream of
tokens to hand back to the compiler to compile into the caller's crate.
Quasi-quoting is a solution to one piece of that -- producing tokens to return
to the compiler.

The idea of quasi-quoting is that we write *code* that we treat as *data*.
Within the `quote!` macro, we can write what looks like code to our text editor
or IDE. We get all the benefits of the editor's brace matching, syntax
highlighting, indentation, and maybe autocompletion. But rather than compiling
that as code into the current crate, we can treat it as data, pass it around,
mutate it, and eventually hand it back to the compiler as tokens to compile into
the macro caller's crate.

This crate is motivated by the procedural macro use case, but is a
general-purpose Rust quasi-quoting library and is not specific to procedural
macros.

*Version requirement: Quote supports any compiler version back to Rust's very
first support for procedural macros in Rust 1.15.0.*

[*Release notes*](https://github.com/dtolnay/quote/releases)

```toml
[dependencies]
quote = "0.6"
```

```rust
#[macro_use]
extern crate quote;
```

## Syntax

The quote crate provides a [`quote!`] macro within which you can write Rust code
that gets packaged into a [`TokenStream`] and can be treated as data. You should
think of `TokenStream` as representing a fragment of Rust source code. This type
can be returned directly back to the compiler by a procedural macro to get
compiled into the caller's crate.

[`TokenStream`]: https://docs.rs/proc-macro2/0.4/proc_macro2/struct.TokenStream.html

Within the `quote!` macro, interpolation is done with `#var`. Any type
implementing the [`quote::ToTokens`] trait can be interpolated. This includes
most Rust primitive types as well as most of the syntax tree types from [`syn`].

[`quote::ToTokens`]: https://docs.rs/quote/0.6/quote/trait.ToTokens.html
[`syn`]: https://github.com/dtolnay/syn

```rust
let tokens = quote! {
    struct SerializeWith #generics #where_clause {
        value: &'a #field_ty,
        phantom: ::std::marker::PhantomData<#item_ty>,
    }

    impl #generics serde::Serialize for SerializeWith #generics #where_clause {
        fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: serde::Serializer,
        {
            #path(self.value, serializer)
        }
    }

    SerializeWith {
        value: #value,
        phantom: ::std::marker::PhantomData::<#item_ty>,
    }
};
```

## Repetition

Repetition is done using `#(...)*` or `#(...),*` similar to `macro_rules!`. This
iterates through the elements of any variable interpolated within the repetition
and inserts a copy of the repetition body for each one. The variables in an
interpolation may be anything that implements `IntoIterator`, including `Vec` or
a pre-existing iterator.

- `#(#var)*` — no separators
- `#(#var),*` — the character before the asterisk is used as a separator
- `#( struct #var; )*` — the repetition can contain other things
- `#( #k => println!("{}", #v), )*` — even multiple interpolations

Note that there is a difference between `#(#var ,)*` and `#(#var),*`—the latter
does not produce a trailing comma. This matches the behavior of delimiters in
`macro_rules!`.

## Examples

### Quoting other quotes

Often you don't want to write your whole `TokenStream` in one piece. The `TokenStream` produced by `quote!{...}` (`syn::export::TokenStream2` not (**!**) `proc_macro::TokenStream`) implements `ToTokens`. Therefore it can be directly quoted.

    let quote1 = quote! {...};
    let quote2 = quote! {...};
    
    let quote_combined = quote!{
        #quote1
        #quote2
    };

### Changing identifiers

Assuming you want to quote an identifier `ident`, but prepended with an underscore. If you naively do:

    quote! {
        _#ident
    }

If `ident` is `foo` this will lead to a space between the underscore and the identifier `_ foo`. You can create a new identifier that the compiler can trace back to the original identifier by creating a new identifier with the span of the previous one.

    fn underscore_ident(ident: &syn::Ident) -> syn::Ident {
        syn::Ident::new(&format!("_{}", ident), ident.span())
    }
    
which you can then use as:

    let underscore_ident = underscore_ident(&ident);
    quote! {
        #underscore_ident
    }
    
### Using `syn::Type`

Say the variable `field_type` contains the `syn::Type` of `Vec<i32>` of a struct_field from your derive macro. Using

    quote!{
        let v: #field_type = some_collection.iter().collect();
    }

will work. However if you want a new vector you'd usually use the turbofish operator `Vec::<i32>::new()`, e.g.

    quote!{
        let v = #field_type::new();
    }

This will expand to `Vec<i32>::new()`, so it won't work. However you can use the field_type directly in `quote!` when you use the fully qualified type notation.

    quote! {
        let v = <#field_type>::new();
    }

You can also use `<Type as Trait>` here, e.g. for the trait MyTrait<T> you could do `<#field_type as MyTrait<#field_type>>`.

## Hygiene

Any interpolated tokens preserve the `Span` information provided by their
`ToTokens` implementation. Tokens that originate within a `quote!` invocation
are spanned with [`Span::call_site()`].

[`Span::call_site()`]: https://docs.rs/proc-macro2/0.4/proc_macro2/struct.Span.html#method.call_site

A different span can be provided explicitly through the [`quote_spanned!`]
macro.

[`quote_spanned!`]: https://docs.rs/quote/0.6/quote/macro.quote_spanned.html

### Limitations

- A non-repeating variable may not be interpolated inside of a repeating block
  ([#7]).
- The same variable may not be interpolated more than once inside of a repeating
  block ([#8]).

[#7]: https://github.com/dtolnay/quote/issues/7
[#8]: https://github.com/dtolnay/quote/issues/8

### Recursion limit

The `quote!` macro relies on deep recursion so some large invocations may fail
with "recursion limit reached" when you compile. If it fails, bump up the
recursion limit by adding `#![recursion_limit = "128"]` to your crate. An even
higher limit may be necessary for especially large invocations. You don't need
this unless the compiler tells you that you need it.

## License

Licensed under either of

 * Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
