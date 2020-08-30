use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{boxed::Box, error::Error, fmt, path::PathBuf};

#[derive(Debug)]
struct SchemaError {
    message: String,
}

impl SchemaError {
    pub fn new(message: &str) -> SchemaError {
        SchemaError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for SchemaError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for SchemaError {}

#[derive(Debug, Deserialize, Serialize)]
pub struct Type {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub description: Option<String>,
    pub fields: Option<Vec<Field>>,
    pub inputs: Option<Vec<Input>>,
    pub interfaces: Option<Vec<TypeRef>>,
    pub enums: Option<Vec<Enum>>,
    #[serde(alias = "possibleTypes")]
    pub possible_types: Option<Vec<TypeRef>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Field {
    pub name: Option<String>,
    pub description: Option<String>,
    pub args: Option<Vec<Input>>,
    #[serde(alias = "type")]
    pub field_type: Option<TypeRef>,
    #[serde(alias = "isDeprecated")]
    pub is_deprecated: Option<bool>,
    #[serde(alias = "deprecationReason")]
    pub deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "type")]
    pub input_type: Option<TypeRef>,
    #[serde(alias = "defaultValue")]
    pub default_value: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Enum {
    name: Option<String>,
    description: Option<String>,
    #[serde(alias = "isDeprecated")]
    is_deprecated: Option<bool>,
    #[serde(alias = "deprecationReason")]
    deprecation_reason: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TypeRef {
    pub name: Option<String>,
    pub kind: Option<String>,
    #[serde(alias = "ofType")]
    pub of_type: Option<Box<TypeRef>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Directive {
    name: Option<String>,
    description: Option<String>,
    locations: Option<Vec<String>>,
    args: Option<Vec<Input>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(alias = "queryType")]
    pub query_type: Option<Type>,
    #[serde(alias = "mutationType")]
    pub mutation_type: Option<Type>,
    #[serde(alias = "subscriptionType")]
    pub subscription_type: Option<Type>,
    pub types: Option<Vec<Type>>,
    pub directives: Option<Vec<Directive>>,
}

impl Schema {
    pub fn from_url(url: &str, headers: &Vec<String>) -> Result<Schema, Box<dyn Error>> {
        let client = Client::new();
        let mut post = client.post(url);
        for header in headers {
            let split: Vec<&str> = header.split(":").collect();
            if split.len() == 2 {
                post = post.header(split[0], split[1]);
            }
        }
        let text = post
            .header("Content-Type", "application/json")
            .body(format!("{{\"query\": \"{}\"}}", SCHEMA_QUERY).replace("\n", ""))
            .send()?
            .text()?;

        Schema::from_str(&text)
    }

    pub fn from_json(_file: &PathBuf, _headers: &Vec<String>) -> Result<Schema, Box<dyn Error>> {
        Err(Box::new(SchemaError::new("not yet implemented")))
    }

    pub fn from_schema(_file: &PathBuf, _headers: &Vec<String>) -> Result<Schema, Box<dyn Error>> {
        Err(Box::new(SchemaError::new("not yet implemented")))
    }

    pub fn from_str(text: &str) -> Result<Schema, Box<dyn Error>> {
        match serde_json::from_str(&text)? {
            Value::Object(map) => match map.get("data") {
                Some(data) => match data.get("__schema") {
                    Some(schema) => {
                        let s: Schema = serde_json::from_str(&schema.to_string())?;
                        return Ok(s);
                    }
                    None => return Err(Box::new(SchemaError::new("schema not in response"))),
                },
                None => return Err(Box::new(SchemaError::new("data not in response"))),
            },
            _ => {
                // I don't think this is reachable; as far as I can tell,
                // serde_json::from_str() fails if text is not a JSON object.
                // You can't pass it an array, for example. So if line 14 passes,
                // we're already guaranteed to have an object.
                return Err(Box::new(SchemaError::new("response format not an object")));
            }
        };
    }

    pub fn get_query_name(&self) -> Option<String> {
        self.query_type.as_ref().and_then(|typ| typ.name.clone())
    }

    pub fn get_mutation_name(&self) -> Option<String> {
        self.mutation_type.as_ref().and_then(|typ| typ.name.clone())
    }

    pub fn get_subscription_name(&self) -> Option<String> {
        self.subscription_type
            .as_ref()
            .and_then(|typ| typ.name.clone())
    }

    pub fn get_type(&self, name: &str) -> Option<&Type> {
        match &self.types {
            Some(types) => {
                for typ in types.iter() {
                    match &typ.name {
                        Some(n) => {
                            if n == name {
                                return Some(&typ);
                            }
                        }
                        None => {}
                    }
                }
                return None;
            }
            None => None,
        }
    }
}

const SCHEMA_QUERY: &str = r#"query IntrospectionQuery {
  __schema {
    queryType {
      name
    }
    mutationType {
      name
    }
    subscriptionType {
      name
    }
    types {
      ...FullType
    }
    directives {
      name
      description
      locations
      args {
        ...InputValue
      }
    }
  }
}

fragment FullType on __Type {
  kind
  name
  description
  fields(includeDeprecated: true) {
    name
    description
    args {
      ...InputValue
    }
    type {
      ...TypeRef
    }
    isDeprecated
    deprecationReason
  }
  inputFields {
    ...InputValue
  }
  interfaces {
    ...TypeRef
  }
  enumValues(includeDeprecated: true) {
    name
    description
    isDeprecated
    deprecationReason
  }
  possibleTypes {
    ...TypeRef
  }
}

fragment InputValue on __InputValue {
  name
  description
  type {
    ...TypeRef
  }
  defaultValue
}

fragment TypeRef on __Type {
  kind
  name
  ofType {
    kind
    name
    ofType {
      kind
      name
      ofType {
        kind
        name
        ofType {
          kind
          name
          ofType {
            kind
            name
            ofType {
              kind
              name
              ofType {
                kind
                name
              }
            }
          }
        }
      }
    }
  }
}"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_pass_when_empty_schema() {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        match Schema::from_str(&response) {
            Err(_) => assert!(false, "schema should parse"),
            Ok(_) => assert!(true),
        }
    }

    #[test]
    fn test_should_fail_when_not_json() {
        let response = "test";
        match Schema::from_str(response) {
            Ok(_) => assert!(false, "plain text should fail"),
            Err(err) => assert_eq!("expected ident at line 1 column 2", err.to_string()),
        }
    }

    #[test]
    fn test_should_fail_when_no_data() {
        let response = r#"{
        }"#;
        match Schema::from_str(&response) {
            Ok(_) => assert!(false, "schema should have data"),
            Err(err) => assert_eq!("data not in response", err.to_string()),
        }
    }

    #[test]
    fn test_should_fail_when_no_schema() {
        let response = r#"{
            "data": {
            }
        }"#;
        match Schema::from_str(&response) {
            Ok(_) => assert!(false, "schema should have __schema"),
            Err(err) => assert_eq!("schema not in response", err.to_string()),
        }
    }

    #[test]
    fn test_should_have_no_query_type_when_none() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.query_type.is_none());
        Ok(())
    }

    #[test]
    fn test_should_have_query_type_when_some() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "queryType": {
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.query_type.is_some());
        Ok(())
    }

    #[test]
    fn test_should_have_query_type_name_when_present() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "queryType": {
                        "name": "Query"
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert_eq!("Query", schema.query_type.unwrap().name.unwrap());
        Ok(())
    }

    #[test]
    fn test_should_return_some_query_name_when_present() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "queryType": {
                        "name": "Query"
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_query_name().is_some());
        assert_eq!("Query", schema.get_query_name().unwrap());
        Ok(())
    }

    #[test]
    fn test_should_return_none_query_name_when_absent() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_query_name().is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_none_query_name_when_name_absent() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "queryType": {
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_query_name().is_none());
        Ok(())
    }

    #[test]
    fn test_should_have_no_mutation_type_when_none() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.mutation_type.is_none());
        Ok(())
    }

    #[test]
    fn test_should_have_mutation_type_when_some() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "mutationType": {
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.mutation_type.is_some());
        Ok(())
    }

    #[test]
    fn test_should_have_mutation_type_name_when_present() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "mutationType": {
                        "name": "mutation"
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert_eq!("mutation", schema.mutation_type.unwrap().name.unwrap());
        Ok(())
    }

    #[test]
    fn test_should_return_some_mutation_name_when_present() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "mutationType": {
                        "name": "mutation"
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_mutation_name().is_some());
        assert_eq!("mutation", schema.get_mutation_name().unwrap());
        Ok(())
    }

    #[test]
    fn test_should_return_none_mutation_name_when_absent() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_mutation_name().is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_none_mutation_name_when_name_absent() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "mutationType": {
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_mutation_name().is_none());
        Ok(())
    }

    #[test]
    fn test_should_have_no_subscription_type_when_none() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.subscription_type.is_none());
        Ok(())
    }

    #[test]
    fn test_should_have_subscription_type_when_some() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "subscriptionType": {
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.subscription_type.is_some());
        Ok(())
    }

    #[test]
    fn test_should_have_subscription_type_name_when_present() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "subscriptionType": {
                        "name": "subscription"
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert_eq!(
            "subscription",
            schema.subscription_type.unwrap().name.unwrap()
        );
        Ok(())
    }

    #[test]
    fn test_should_return_some_subscription_name_when_present() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "subscriptionType": {
                        "name": "subscription"
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_subscription_name().is_some());
        assert_eq!("subscription", schema.get_subscription_name().unwrap());
        Ok(())
    }

    #[test]
    fn test_should_return_none_subscription_name_when_absent() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_subscription_name().is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_none_subscription_name_when_name_absent() -> Result<(), Box<dyn Error>> {
        let response = r#"{
            "data": {
                "__schema": {
                    "subscriptionType": {
                    }
                }
            }
        }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_subscription_name().is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_none_when_no_types() -> Result<(), Box<dyn Error>> {
        let response = r#"{
        "data": {
            "__schema": {
            }
        }
    }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_type("hello").is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_none_when_type_has_no_name() -> Result<(), Box<dyn Error>> {
        let response = r#"{
        "data": {
            "__schema": {
                "types": [
                    {
                        "kind": "SCALAR"
                    }
                ]
            }
        }
    }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_type("hello").is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_none_when_no_type_of_name() -> Result<(), Box<dyn Error>> {
        let response = r#"{
        "data": {
            "__schema": {
                "types": [
                    {
                        "name": "you're not my"
                    }
                ]
            }
        }
    }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_type("hello").is_none());
        Ok(())
    }

    #[test]
    fn test_should_return_some_when_type_of_name() -> Result<(), Box<dyn Error>> {
        let response = r#"{
        "data": {
            "__schema": {
                "types": [
                    {
                        "name": "you're not my"
                    }
                ]
            }
        }
    }"#;
        let schema = Schema::from_str(&response)?;
        assert!(schema.get_type("you're not my").is_some());
        Ok(())
    }
}
