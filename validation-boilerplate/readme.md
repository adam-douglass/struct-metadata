# Validation Boilerplate

Utility for generating boilerplate for types that require runtime validation whenever they are loaded.

This generates a private type that uses derive(Deserialize) as normal then passes it to a validation function to be converted to the user type.

It can be applied recursively and is compatable with any type that implements deserialize normally.
