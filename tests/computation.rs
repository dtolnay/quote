#[macro_use]
extern crate quote;

pub use quote::Tokens;

pub struct SimpleStruct1D<T> {
    x: T,
}

pub struct SimpleStruct2D<T> {
    x: T,
    y: T,
}

mod simple {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn array_access() {
        let a = [quote!(zero), quote!(one)];
        let b = [&a, &a];
        let tokens = quote!(#{a[0]} - #{a[1]} - #{b[1][0]} - #{b[0][1]});

        assert_eq!(tokens, quote!(
            zero - one - zero - one
        ));
    }

    #[test]
    fn tuple_access() {
        let a = (quote!(zero), quote!(one));
        let b = (&a, &a);
        let tokens = quote!(#{a.0} - #{a.1} - #{(b.1).0} - #{(b.0).1});

        assert_eq!(tokens, quote!(
            zero - one - zero - one
        ));
    }

    #[test]
    fn struct_access() {
        let a = SimpleStruct2D {
            x: quote!(zero),
            y: quote!(one),
        };
        let b = SimpleStruct2D {
            x: &a,
            y: &a,
        };
        let tokens = quote!(#{a.x} - #{a.y} - #{b.y.x} - #{b.x.y});

        assert_eq!(tokens, quote!(
            zero - one - zero - one
        ));
    }

    #[test]
    fn function_call() {
        let a = || { quote!(Lorem) };
        let b = (&a,);
        let c = [&a];
        let tokens = quote!(#{a()} - #{(b.0)()} - #{c[0]()});

        assert_eq!(tokens, quote!(
            Lorem - Lorem - Lorem
        ));
    }
}

mod mixed {
    #[allow(unused_imports)]
    use super::*;

    #[test]
    fn leading_array_access() {
        let tuple = [(quote!(tuple),)];
        let strukt = [SimpleStruct1D { x: quote!(struct) }];
        let func = [|| { quote!(func) }];
        let tokens = quote!(#{tuple[0].0} - #{strukt[0].x} - #{func[0]()});

        assert_eq!(tokens, quote!(
            tuple - struct - func
        ));
    }

    #[test]
    fn leading_tuple_access() {
        let strukt = (SimpleStruct1D { x: quote!(struct) },);
        let func = (|| { quote!(func) },);
        let array = ([quote!(array)],);
        let tokens = quote!(#{(strukt.0).x} - #{(func.0)()} - #{(array.0)[0]});

        assert_eq!(tokens, quote!(
            struct - func - array
        ));
    }

    #[test]
    fn leading_struct_access() {
        struct MixedStruct {
            array: Vec<Tokens>,
            tuple: (Tokens,),
        }

        impl MixedStruct {
            fn func(&self) -> Tokens {
                quote!(func)
            }
        }

        let a = MixedStruct {
            array: vec![quote!(array)],
            tuple: (quote!(tuple),),
        };
        let tokens = quote!(#{a.func()} - #{a.array[0]} - #{a.tuple.0});

        assert_eq!(tokens, quote!(
            func - array - tuple
        ));
    }

    #[test]
    fn leading_function_call() {
        let array = || { [quote!(array)] };
        let tuple = || { (quote!(tuple),) };
        let strukt = || { SimpleStruct1D { x: quote!(struct) } };
        let tokens = quote!(#{array()[0]} - #{tuple().0} - #{strukt().x});

        assert_eq!(tokens, quote!(
            array - tuple - struct
        ));
    }
}
