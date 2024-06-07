use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, quote_spanned};

#[test]
fn test_quote_not_block() {
    // ensure "foo" is not executed
    let _ = quote! {
        #{ foo }
        #@{ foo }#
        #{ foo }@
        @{ foo }@
    };
}

#[test]
fn test_quote_block() {
    let variants = vec![quote! { A }, quote! { B }, quote! { C }];

    let tokens = quote! {
        match x { #@{
            variants.iter().enumerate().map(|(i, v)| {
                quote! { Self::#v => #i, }
            })
        }@ }
    };

    let expected = concat!(
        "match x { ",
        "Self :: A => 0usize , ",
        "Self :: B => 1usize , ",
        "Self :: C => 2usize , ",
        "}"
    );

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_expressions() {
    let a = quote! { test_a };
    let b = quote! { test_b };
    let c = quote! { test_c };
    let v = vec![&a, &b, &c];

    let tokens = quote! {
        #a
        #@{ v[0] }@
        #b
        #@{ v[1] }@
        #c
    };

    let expected = "test_a test_a test_b test_b test_c";

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_expressions_chaining() {
    let a = quote! { test_a };
    let b = quote! { test_b };
    let c = quote! { test_c };
    let v = vec![&a, &b, &c];

    let tokens = quote! {
        #a
        #@{ v[0] }@
        #b
        #@{ v[1] }@
        #c
    };

    let expected = "test_a test_a test_b test_b test_c";

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_spanned() {
    let x = 42;
    let span = Span::call_site();
    let tokens = quote_spanned! { span =>
        #@{
            if x == 42 {
                quote_spanned! { span => true }
            } else {
                quote_spanned! { span => false }
            }
        }@
    };
    assert_eq!("true", tokens.to_string());
    let tokens = quote! {
        #@{
            if x != 42 {
                quote_spanned! { span => true }
            } else {
                quote_spanned! { span => false }
            }
        }@
    };
    assert_eq!("false", tokens.to_string());
    let tokens = quote! {
        #@{
            if x - 42 == 0 {
                quote! { true }
            } else {
                quote_spanned! { span => false }
            }
        }@
    };
    assert_eq!("true", tokens.to_string());
}

#[test]
fn test_quote_block_with_ident() {
    let variants = vec![quote! { A }, quote! { B }, quote! { C }];

    let before = quote! { do_before() };
    let call = quote! { do_foo };
    let after = quote! { do_after() };

    let tokens = quote! {
        #before
        #@{
            let mut t = TokenStream::new();
            for v in variants {
                t.extend(quote! { #call(#v); });
            }
            t
        }@
        #after
    };

    let expected = concat!(
        "do_before () ",
        "do_foo (A) ; do_foo (B) ; do_foo (C) ; ",
        "do_after ()"
    );

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_nested_simple() {
    let foo = quote! { foo };
    let bar = quote! { bar };

    let tokens = quote! {
        #@{
            quote!{ #foo #bar }
        }@
    };
    let expected = "foo bar";
    assert_eq!(expected, tokens.to_string());

    let tokens = quote! {
        #foo
        #@{
            quote!{ #foo #@{ quote!{ #foo #bar } }@ #bar }
        }@
        #bar
    };
    let expected = "foo foo foo bar bar bar";
    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_nested() {
    let structs = vec![
        (
            quote! { StructA },
            vec![quote! { foo }, quote! { bar }],
            vec![quote! { FieldA1 }, quote! { FieldA2 }],
        ),
        (
            quote! { StructB },
            vec![quote! { foo }, quote! { bar }, quote! { baz }],
            vec![quote! { FieldB1 }, quote! { FieldB2 }, quote! { FieldB3 }],
        ),
        (quote! { StructC }, vec![], vec![]),
    ];

    let mod_ident = quote! { block_test };

    let tokens = quote! {
        mod #mod_ident { #@{
            structs.iter().map(|(s, f, f_ty)| {
                quote! {
                    struct #s {
                        #@{ {{ // these braces should have no effect, just making sure the matching
                            // works
                            std::iter::zip(f.iter(), f_ty.iter()).map(|(f, f_ty)| {
                                quote! { #f: #f_ty, }
                            })
                        }} }@
                    }
                }
            })
        }@ }
    };

    let expected = concat!(
        "mod block_test { ",
        "struct StructA { foo : FieldA1 , bar : FieldA2 , } ",
        "struct StructB { foo : FieldB1 , bar : FieldB2 , baz : FieldB3 , } ",
        "struct StructC { } ",
        "}"
    );

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_fn_call() {
    let lits = ["a", "b", "c"];
    fn make_my_ident(lit: &str) -> Ident {
        format_ident!("my_ident_{}", lit)
    }
    let tokens = quote! {
        #@{ make_my_ident("foo") }@
        #@{
            lits.iter().map(|x| make_my_ident(*x))
        }@
        #@{ make_my_ident("bar") }@
    };

    let expected = "my_ident_foo my_ident_a my_ident_b my_ident_c my_ident_bar";
    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_with_iter() {
    let lits = vec![quote! { a }, quote! { b }, quote! { c }];

    let tokens = quote! {
        #( invoke( #lits ); )*
        #@{ lits.iter() }@
        #@{ lits.iter().map(|x| quote! { invoke(#x) }) }@
    };

    let expected = concat!(
        "invoke (a) ; invoke (b) ; invoke (c) ; ",
        "a b c ",
        "invoke (a) invoke (b) invoke (c)"
    );
    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_nested_iter() {
    let a = quote! { a };
    let b = quote! { b };
    let c = quote! { c };
    let ids = [&a, &b, &c];

    let tokens = quote! {
        #(
            invoke(#ids);
            #@{
                quote!{#ids}.to_string()
            }@
            #c
        )*
    };

    let expected = concat!(
        "invoke (a) ; ",
        "\"a\" ",
        "c ",
        "invoke (b) ; ",
        "\"b\" ",
        "c ",
        "invoke (c) ; ",
        "\"c\" ",
        "c",
    );

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_surround_regular_block() {
    let ident = quote! { my_ident };

    let tokens = quote! {
        { hello }#@
        {
            Ident::new("mystery", Span::call_site())
        }
        @{ world }#
        ident
    };

    let expected = "{ hello } mystery { world } my_ident";
    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_many_nesting() {
    let a = quote! { a };
    let b = quote! { b };
    let ids = vec![&a, &b];
    let ids2 = vec![vec![&a, &b], vec![&b, &a]];

    let tokens = quote! {
        #(
            invoke(#ids);
            #@{
                quote! { #( #@{ quote!(#ids) }@ (#ids2); )* }
            }@
        )*
    };

    let expected = concat!(
        "invoke (a) ; ",
        "a (a) ; a (b) ; ",
        "invoke (b) ; ",
        "b (b) ; b (a) ;"
    );

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_quote_block_bind_iter() {
    let a = quote! { a };
    let b = quote! { b };
    let ids2 = vec![vec![&a, &b], vec![&b, &a]];

    let tokens = quote! {
        #(
            invoke(#(#ids2),*);
            #@{
                let mut t = TokenStream::new();
                t.extend(ids2[0].clone());
                t.extend(quote! { + });
                t.extend(ids2[1].clone());
                t
            }@
        )*
    };

    let expected = "invoke (a , b) ; a + b invoke (b , a) ; b + a";

    assert_eq!(expected, tokens.to_string());
}
