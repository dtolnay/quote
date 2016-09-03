#[macro_use]
extern crate quote;

#[test]
fn test_quote_impl() {
    let tokens = quote!(
        impl<'a, T: ToTokens> ToTokens for &'a T {
            fn to_tokens(&self, tokens: &mut Tokens) {
                (**self).to_tokens(tokens)
            }
        }
    );

    let expected = concat!(
        "impl < 'a , T : ToTokens > ToTokens for & 'a T { ",
            "fn to_tokens ( & self , tokens : & mut Tokens ) { ",
                "( * * self ) . to_tokens ( tokens ) ",
            "} ",
        "} "
    );

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_substitution() {
    let n = 1;
    let tokens = quote!(#n <#n> (#n) [#n] {#n});

    let expected = "1 < 1 > ( 1 ) [ 1 ] { 1 } ";

    assert_eq!(expected, tokens.to_string());
}

#[test]
fn test_iter() {
    let primes = vec![2, 3, 5, 7];

    assert_eq!("2 3 5 7 ", quote!(#(primes)*).to_string());

    assert_eq!("2 , 3 , 5 , 7 , ", quote!(#(primes,)*).to_string());

    assert_eq!("2 , 3 , 5 , 7 ", quote!(#(primes),*).to_string());
}

#[test]
fn test_advanced() {
    let generics = quote!( <'a, T> );

    let where_clause = quote!( where T: Serialize );

    let field_ty = quote!( String );

    let item_ty = quote!( Cow<'a, str> );

    let path = quote!( SomeTrait::serialize_with );

    let value = quote!( self.x );

    let tokens = quote! {
        struct SerializeWith #generics #where_clause {
            value: &'a #field_ty,
            phantom: ::std::marker::PhantomData<#item_ty>,
        }

        impl #generics ::serde::Serialize for SerializeWith #generics #where_clause {
            fn serialize<S>(&self, s: &mut S) -> Result<(), S::Error>
                where S: ::serde::Serializer
            {
                #path(self.value, s)
            }
        }

        SerializeWith {
            value: #value,
            phantom: ::std::marker::PhantomData::<#item_ty>,
        }
    };

    let expected = concat!(
        "struct SerializeWith < 'a , T >  where T : Serialize  { ",
            "value : & 'a String  , ",
            "phantom : :: std :: marker :: PhantomData < Cow < 'a , str >  > , ",
        "} ",
        "impl < 'a , T >  :: serde :: Serialize for SerializeWith < 'a , T >  where T : Serialize  { ",
            "fn serialize < S > ( & self , s : & mut S ) -> Result < ( ) , S :: Error > ",
                "where S : :: serde :: Serializer ",
            "{ ",
                "SomeTrait :: serialize_with  ( self . value , s ) ",
            "} ",
        "} ",
        "SerializeWith { ",
            "value : self . x  , ",
            "phantom : :: std :: marker :: PhantomData :: < Cow < 'a , str >  > , ",
        "} "
    );

    assert_eq!(expected, tokens.to_string());
}
