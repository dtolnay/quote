use super::Tokens;

pub trait ToTokens {
    fn to_tokens(&self, &mut Tokens);
}

impl<'a, T: ToTokens> ToTokens for &'a T {
    fn to_tokens(&self, tokens: &mut Tokens) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Box<T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        (**self).to_tokens(tokens);
    }
}

impl<T: ToTokens> ToTokens for Option<T> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        if let Some(ref t) = *self {
            t.to_tokens(tokens);
        }
    }
}

impl ToTokens for str {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(&format!("{:?}", self));
    }
}

impl ToTokens for String {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(&format!("{:?}", self));
    }
}

#[derive(Debug)]
pub struct ByteStr<'a>(pub &'a str);

impl<'a> ToTokens for ByteStr<'a> {
    fn to_tokens(&self, tokens: &mut Tokens) {
        tokens.append(&format!("b{:?}", self.0));
    }
}

macro_rules! impl_to_tokens_display {
    ($ty:ty) => {
        impl ToTokens for $ty {
            fn to_tokens(&self, tokens: &mut Tokens) {
                tokens.append(&self.to_string());
            }
        }
    };
}

impl_to_tokens_display!(Tokens);
impl_to_tokens_display!(bool);

macro_rules! impl_to_tokens_integer {
    ($ty:ty) => {
        impl ToTokens for $ty {
            fn to_tokens(&self, tokens: &mut Tokens) {
                tokens.append(&format!(concat!("{}", stringify!($ty)), self));
            }
        }
    };
}

impl_to_tokens_integer!(i8);
impl_to_tokens_integer!(i16);
impl_to_tokens_integer!(i32);
impl_to_tokens_integer!(i64);
impl_to_tokens_integer!(isize);
impl_to_tokens_integer!(u8);
impl_to_tokens_integer!(u16);
impl_to_tokens_integer!(u32);
impl_to_tokens_integer!(u64);
impl_to_tokens_integer!(usize);

macro_rules! impl_to_tokens_floating {
    ($ty:ty) => {
        impl ToTokens for $ty {
            fn to_tokens(&self, tokens: &mut Tokens) {
                use std::num::FpCategory::*;
                match self.classify() {
                    Zero | Subnormal | Normal => {
                        tokens.append(&format!(concat!("{}", stringify!($ty)), self));
                    }
                    Nan => {
                        tokens.append("::");
                        tokens.append("std");
                        tokens.append("::");
                        tokens.append(stringify!($ty));
                        tokens.append("::");
                        tokens.append("NAN");
                    }
                    Infinite => {
                        tokens.append("::");
                        tokens.append("std");
                        tokens.append("::");
                        tokens.append(stringify!($ty));
                        tokens.append("::");
                        if self.is_sign_positive() {
                            tokens.append("INFINITY");
                        } else {
                            tokens.append("NEG_INFINITY");
                        }
                    }
                }
            }
        }
    };
}
impl_to_tokens_floating!(f32);
impl_to_tokens_floating!(f64);
