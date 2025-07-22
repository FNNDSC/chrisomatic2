use compact_str::CompactString;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PluginSpec {
    name: CompactString,
    version: Option<CompactString>,
}

impl serde::ser::Serialize for PluginSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        todo!()
    }
}

impl<'de> serde::de::Deserialize<'de> for PluginSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        todo!()
    }
}
