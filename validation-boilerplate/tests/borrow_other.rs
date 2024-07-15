use std::collections::HashSet;
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
#[validated_deserialize(Config)]
struct Container<'a, 'b> {
    #[validate]
    config: ValidatedType,
    other: &'a str,
    also: &'b str,
    normal_data: u64
}

/// make sure both the container and unvalidated container have the same lifetime constraints
#[allow(dead_code)]
struct TestLifetimesSame<'a, 'b> {
    a: Container<'a, 'b>,
    b: ContainerUnvalidated<'a, 'b>,
}


#[test]
fn test_load() {
    let config = Config{
        valid_strings: ["cats".to_owned()].into_iter().collect(),
    };

    // clean load
    let mut deserializer = serde_json::Deserializer::from_str(r#"{
        "config": "cats",
        "other": "dogs",
        "also": "bird",
        "normal_data": 10
    }"#);
    let container = Container::deserialize_and_validate(&mut deserializer, &config).unwrap();
    assert_eq!(container, Container{ config: ValidatedType("cats".to_owned()), other: "dogs", also: "bird", normal_data: 10 });

    // refuse during validation
    let mut deserializer = serde_json::Deserializer::from_str(r#"{
        "config": "dogs",
        "other": "dogs",
        "also": "bird",
        "normal_data": 10
    }"#);
    assert!(Container::deserialize_and_validate(&mut deserializer, &config).is_err());

}