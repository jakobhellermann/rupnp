use std::fmt;

pub enum Bool {
    False,
    True,
}

impl fmt::Display for Bool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Bool::False => write!(f, "0"),
            Bool::True => write!(f, "1"),
        }
    }
}

impl Into<bool> for Bool {
    fn into(self) -> bool {
        match self {
            Bool::False => false,
            Bool::True => true,
        }
    }
}

impl From<bool> for Bool {
    fn from(b: bool) -> Self {
        if b {
            Bool::True
        } else {
            Bool::False
        }
    }
}

#[derive(Debug)]
pub struct ParseBoolError {
    _priv: (),
}
impl fmt::Display for ParseBoolError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "provided string was not `0` or `1`".fmt(f)
    }
}
impl std::error::Error for ParseBoolError {}

impl std::str::FromStr for Bool {
    type Err = ParseBoolError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "0" => Ok(Bool::False),
            "1" => Ok(Bool::True),
            _ => Err(ParseBoolError { _priv: () }),
        }
    }
}
