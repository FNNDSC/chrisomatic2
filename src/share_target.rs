/// User or group to share a ChRIS resource with.
#[derive(Debug, Clone, PartialEq)]
pub enum ShareTarget {
    /// Username to share with
    User(String),
    /// Group to share with
    Group(String),
}

impl serde::ser::Serialize for ShareTarget {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ShareTarget::User(username) => serializer.serialize_str(username),
            ShareTarget::Group(group) => serializer.serialize_str(&format!("group:{group}")),
        }
    }
}

impl<'de> serde::de::Deserialize<'de> for ShareTarget {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(StringVisitor)
    }
}

struct StringVisitor;

impl<'de> serde::de::Visitor<'de> for StringVisitor {
    type Value = ShareTarget;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("prefix \"user\" or \"group\" before ':' character")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        if let Some((l, r)) = v.split_once(':') {
            match l {
                "group" => Ok(ShareTarget::Group(r.to_string())),
                "user" => Ok(ShareTarget::User(r.to_string())),
                l => Err(E::invalid_value(serde::de::Unexpected::Str(l), &self)),
            }
        } else {
            Ok(ShareTarget::User(v.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("alice", ShareTarget::User("alice".to_string()))]
    #[case("user:alice", ShareTarget::User("alice".to_string()))]
    #[case("group:friends", ShareTarget::Group("friends".to_string()))]
    fn test_deserialize(#[case] input: &str, #[case] expected: ShareTarget) {
        let value = toml::Value::String(input.to_string());
        let actual = value.try_into();
        assert_eq!(actual, Ok(expected))
    }

    #[rstest]
    fn test_deserialize_error() {
        let value = toml::Value::String("invalid:something".to_string());
        let actual: Result<ShareTarget, _> = value.try_into();
        assert!(actual.is_err())
    }

    #[rstest]
    #[case(ShareTarget::User("alice".to_string()), "alice")]
    #[case(ShareTarget::Group("friends".to_string()), "group:friends")]
    fn test_serialize(#[case] input: ShareTarget, #[case] expected: &str) {
        let actual = toml::Value::try_from(input);
        assert_eq!(actual, Ok(toml::Value::String(expected.to_string())))
    }
}
