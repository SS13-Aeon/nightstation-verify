use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ckey(String);

fn filter_ckey(key: &str) -> String {
    key.chars()
        .filter(|c| c.is_ascii_alphanumeric())
        .map(|c| c.to_ascii_lowercase())
        .collect()
}

impl Ckey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for Ckey {
    fn from(value: &str) -> Self {
        Self(filter_ckey(value))
    }
}

impl From<&String> for Ckey {
    fn from(value: &String) -> Self {
        Self(filter_ckey(value))
    }
}

impl fmt::Display for Ckey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}
