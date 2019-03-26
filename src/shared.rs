use serde::Deserialize;

#[derive(Deserialize, Default)]
pub struct Value<T> where T: Default {
    #[serde(default = "Default::default")]
    #[serde(rename = "$value")]
    pub value: T,
}
impl<T: Default> std::fmt::Debug for Value<T>
where
    T: std::fmt::Debug,
{
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
    pub fn minor(&self) -> u32 {
        self.minor
    }
}
