use std::collections::HashSet;
use validation_boilerplate::ValidatedDeserialize;

struct Config {
    valid_strings: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedType<'a>(&'a str);

impl<'de> ValidatedDeserialize<'de, Config> for ValidatedType<'de> {
    type ProxyType = &'de str;

    fn validate(input: Self::ProxyType, validator: &Config) -> Result<Self, String> {
        if validator.valid_strings.contains(input) {
            Ok(Self(input))
        } else {
            Err(format!("Invalid value: {input}"))
        }
    }
}

#[derive(Debug, ValidatedDeserialize, PartialEq, Eq)]
#[validated_deserialize(Config)]
struct Container<'a> (#[validate] ValidatedType<'a>, &'a str);

#[test]
fn test_load() {
    let config = Config{
        valid_strings: ["cats".to_owned()].into_iter().collect(),
    };

    // clean load
    let mut deserializer = serde_json::Deserializer::from_str(r#"["cats", "dogs"]"#);
    let container = Container::deserialize_and_validate(&mut deserializer, &config).unwrap();
    assert_eq!(container, Container(ValidatedType("cats"), "dogs"));

    // refuse during validation
    let mut deserializer = serde_json::Deserializer::from_str(r#"["dogs", "dogs"]"#);
    assert!(Container::deserialize_and_validate(&mut deserializer, &config).is_err());

}