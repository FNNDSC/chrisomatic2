use std::str::FromStr;

use compact_str::{CompactString, ToCompactString};
use serde::Deserialize;

/// A specified [_ChRIS_ plugin](https://chrisproject.org/docs/plugins).
///
/// A plugin may be specified by either:
///
/// - `{name}`, e.g. `pl-dcm2niix` specifies to use any version
/// - `{name}@{version}`, e.g. `pl-dcm2niix` specifies to use v0.1.0
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct PluginSpec {
    pub name: CompactString,
    pub version: Option<CompactString>,
}

impl ToCompactString for PluginSpec {
    fn to_compact_string(&self) -> CompactString {
        let mut value = self.name.clone();
        if let Some(version) = &self.version {
            value.push('@');
            value.push_str(version);
        }
        value
    }

    fn try_to_compact_string(&self) -> Result<CompactString, compact_str::ToCompactStringError> {
        Ok(self.to_compact_string())
    }
}

impl PluginSpec {
    /// Specify plugin with specific version.
    #[inline(always)]
    pub fn new(name: impl Into<CompactString>, version: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            version: Some(version.into()),
        }
    }

    /// Specify plugin by name with any version.
    #[inline(always)]
    pub fn from_name(name: impl Into<CompactString>) -> Self {
        Self {
            name: name.into(),
            version: None,
        }
    }
}

impl std::str::FromStr for PluginSpec {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let value = if let Some((l, r)) = s.split_once('@') {
            PluginSpec::new(l, r)
        } else {
            PluginSpec::from_name(s)
        };
        Ok(value)
    }
}

impl<'a> Deserialize<'a> for PluginSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'a>,
    {
        deserializer.deserialize_str(Visitor)
    }
}

struct Visitor;

impl<'a> serde::de::Visitor<'a> for Visitor {
    type Value = PluginSpec;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("invalid PluginSpec")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(PluginSpec::from_str(v).unwrap())
    }
}

impl serde::ser::Serialize for PluginSpec {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_compact_string().as_str())
    }
}

impl From<crate::PluginConfig> for PluginSpec {
    fn from(value: crate::PluginConfig) -> Self {
        PluginSpec {
            name: value.name,
            version: value.version,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;
    use rstest::*;

    #[rstest]
    #[case("pl-dcm2niix", PluginSpec::from_name("pl-dcm2niix"))]
    #[case("pl-dcm2niix@1.2.3", PluginSpec::new("pl-dcm2niix", "1.2.3"))]
    fn test_plugin_spec(#[case] input: &str, #[case] expected: PluginSpec) {
        let actual = PluginSpec::from_str(input).unwrap();
        assert_eq!(actual, expected);
        assert_eq!(&actual.to_compact_string(), input);

        let value = serde_json::Value::String(input.to_string());
        let deser: PluginSpec = serde_json::from_value(value).unwrap();
        assert_eq!(deser, expected);
        let sered = serde_json::to_string(&deser).unwrap();
        assert_eq!(sered, format!("\"{input}\""));
    }
}
