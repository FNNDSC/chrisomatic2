pub(crate) const SAMPLE_TOML: &'static str = include_str!("./sample.toml");

#[cfg(test)]
mod tests {
    use chrisomatic_spec::GivenManifest;

    use super::*;

    #[test]
    fn test_sample_is_valid() {
        let _sample: GivenManifest = toml::from_str(SAMPLE_TOML).unwrap();
    }
}
