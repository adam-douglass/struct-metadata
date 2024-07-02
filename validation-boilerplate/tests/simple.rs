use std::collections::HashSet;
use validation_boilerplate::ValidatedDeserialize;

struct Config {
    valid_strings: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedType(String);

impl<'de> ValidatedDeserialize<'de, Config> for ValidatedType {
    type ProxyType = String;

    fn validate(input: Self::ProxyType, validator: &Config) -> Result<Self, String> {
        if validator.valid_strings.contains(&input) {
            Ok(Self(input))
        } else {
            Err(format!("Invalid value: {input}"))
        }
    }
}

#[derive(Debug, ValidatedDeserialize, PartialEq, Eq)]
#[validated_deserialize(Config)]
struct Container {
    #[validate]
    config: ValidatedType,
    // pair: Pair,
    normal_data: u64
}

#[test]
fn test_load() {
    let config = Config{
        valid_strings: ["cats".to_owned()].into_iter().collect(),
    };

    // clean load
    let mut deserializer = serde_json::Deserializer::from_str(r#"{
        "config": "cats",
        "normal_data": 10
    }"#);
    let container = Container::deserialize_and_validate(&mut deserializer, &config).unwrap();
    assert_eq!(container, Container{ config: ValidatedType("cats".to_owned()), normal_data: 10 });

    // refuse during validation
    let mut deserializer = serde_json::Deserializer::from_str(r#"{
        "config": "dogs",
        "normal_data": 10
    }"#);
    assert!(Container::deserialize_and_validate(&mut deserializer, &config).is_err());

}