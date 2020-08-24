use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::boxed::Box;
use std::error::Error;
use std::fmt;

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

impl Error for SchemaError {
    fn description(&self) -> &str {
        &self.message
    }
}

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

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{}", name),
            None => write!(f, ""),
        }
    }
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
    pub subscription: Option<Type>,
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
        return Schema::from_str(&text);
    }

    pub fn from_str(text: &str) -> Result<Schema, Box<dyn Error>> {
        let json: Value = serde_json::from_str(&text)?;
        match json {
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
        self.subscription.as_ref().and_then(|typ| typ.name.clone())
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
            Ok(_) => assert!(true),
            Err(_) => assert!(false, "schema should parse"),
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
}
