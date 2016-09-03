use std::fmt::{self, Display};

#[derive(Debug)]
pub struct Tokens(String);

impl Tokens {
    pub fn new() -> Self {
        Tokens(String::new())
    }

    pub fn append(&mut self, token: &str) {
        self.0.push_str(token);
        self.0.push(' ');
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
