use super::ToTokens;
use std::fmt::{self, Display};

#[derive(Debug, Clone)]
pub struct Tokens(String);

impl Tokens {
    pub fn new() -> Self {
        Tokens(String::new())
    }

    pub fn append(&mut self, token: &str) {
        self.0.push_str(token);
        self.0.push(' ');
    }

    pub fn append_separated<T, I>(&mut self, iter: I, sep: &str)
        where T: ToTokens,
              I: IntoIterator<Item = T>
    {
        for (i, token) in iter.into_iter().enumerate() {
            if i > 0 {
                self.append(sep);
            }
            token.to_tokens(self);
        }
    }
}

impl Default for Tokens {
    fn default() -> Self {
        Tokens::new()
    }
}

impl Display for Tokens {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        self.0.fmt(formatter)
    }
}
