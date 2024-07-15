use std::collections::HashSet;
use serde::Deserialize;
use validation_boilerplate::ValidatedDeserialize;

struct Config {
    valid_strings: HashSet<String>,
}

#[derive(Debug, PartialEq, Eq)]
struct ValidatedType(String);

impl From<ValidatedType> for String {
    fn from(val: ValidatedType) -> Self {
        val.0
    }
}

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
#[validated_deserialize(Config, derive=(Debug, PartialEq, Eq))]
struct Container {
    #[validate]
    config: ValidatedType,
    // pair: Pair,
    normal_data: u64
}

/// make sure both the container and unvalidated container have the same lifetime constraints
#[allow(dead_code)]
struct TestLifetimesSame {
    a: Container,
    b: ContainerUnvalidated,
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

    // Load without validation
    let mut deserializer = serde_json::Deserializer::from_str(r#"{
        "config": "dogs",
        "normal_data": 10
    }"#);
    let loaded = ContainerUnvalidated::deserialize(&mut deserializer).unwrap();
    assert_eq!(loaded, ContainerUnvalidated{config: "dogs".to_owned(), normal_data: 10});
}