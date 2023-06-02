use base64::Engine;
use serde::de::Error as _;
use serde::{Deserialize, Deserializer};
use std::fmt::{Debug, Formatter};

#[derive(Clone, PartialEq, Default)]
pub struct InfoName(Vec<u8>);

impl InfoName {
    pub fn as_str(&self) -> &str {
        unsafe { std::str::from_utf8_unchecked(&self.0) }
    }
}

impl PartialEq<str> for InfoName {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl std::fmt::Display for InfoName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl Debug for InfoName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.as_str())
    }
}

impl AsRef<str> for InfoName {
    fn as_ref(&self) -> &str {
        std::str::from_utf8(&self.0).unwrap()
    }
}

impl<'de> Deserialize<'de> for InfoName {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let v: String = Deserialize::deserialize(deserializer)?;
        Ok(InfoName(
            ::base64::engine::general_purpose::STANDARD
                .decode(v)
                .map_err(|err| D::Error::custom(err.to_string()))?,
        ))
    }
}
