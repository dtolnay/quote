use super::Tokens;

pub trait ToTokens {
    fn to_tokens(&self, &mut Tokens);
}

impl<'a, T: ToTokens> ToTokens for &'a T {
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
impl_to_tokens_display!(i8);
impl_to_tokens_display!(i16);
impl_to_tokens_display!(i32);
impl_to_tokens_display!(i64);
impl_to_tokens_display!(isize);
impl_to_tokens_display!(u8);
impl_to_tokens_display!(u16);
impl_to_tokens_display!(u32);
impl_to_tokens_display!(u64);
impl_to_tokens_display!(usize);
impl_to_tokens_display!(f32);
impl_to_tokens_display!(f64);
