use serde::{de, Deserialize};
use ssdp_client::search::URN;

#[derive(Deserialize, Default)]
pub struct Value<T>
where
    T: Default,
{
    #[serde(default = "Default::default")]
    #[serde(rename = "$value")]
    pub value: T,
}
impl<T: Default + std::fmt::Debug> std::fmt::Debug for Value<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", &self.value)
    }
}

#[derive(Deserialize, Debug)]
pub struct SpecVersion {
    major: u32,
    minor: u32,
}
impl SpecVersion {
    pub fn major(&self) -> u32 {
        self.major
    }
    #[allow(unused)]
    pub fn minor(&self) -> u32 {
        self.minor
    }
}

pub(crate) fn deserialize_urn<'de, D>(deserializer: D) -> Result<URN<'static>, D::Error>
where
    D: de::Deserializer<'de>,
{
    struct STVisitor;
    impl<'de> de::Visitor<'de> for STVisitor {
        type Value = URN<'static>;
        fn expecting(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(f, "a string containing a SearchTarget")
        }
        fn visit_str<E: de::Error>(self, v: &str) -> Result<Self::Value, E> {
            v.parse().map_err(E::custom)
        }
        fn visit_map<M: de::MapAccess<'de>>(self, mut access: M) -> Result<Self::Value, M::Error> {
            use de::Error;
            for (key, value) in access.next_entry::<String, String>()? {
                if key == "$value" {
                    return value.parse().map_err(M::Error::custom);
                }
            }
            Err(de::Error::missing_field("$value"))
        }
    }

    deserializer.deserialize_any(STVisitor)
}
