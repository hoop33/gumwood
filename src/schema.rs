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
    name: Option<String>,
    kind: Option<String>,
    description: Option<String>,
    fields: Option<Vec<Field>>,
    inputs: Option<Vec<Input>>,
    interfaces: Option<Vec<TypeRef>>,
    enums: Option<Vec<Enum>>,
    #[serde(alias = "possibleTypes")]
    possible_types: Option<Vec<TypeRef>>,
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
    name: Option<String>,
    description: Option<String>,
    args: Option<Vec<Input>>,
    #[serde(alias = "type")]
    field_type: Option<TypeRef>,
    #[serde(alias = "isDeprecated")]
    is_deprecated: Option<bool>,
    #[serde(alias = "deprecationReason")]
    deprecation_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Input {
    name: Option<String>,
    description: Option<String>,
    #[serde(alias = "type")]
    input_type: Option<TypeRef>,
    #[serde(alias = "defaultValue")]
    default_value: Option<String>,
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
    name: Option<String>,
    kind: Option<String>,
    #[serde(alias = "ofType")]
    of_type: Option<Box<TypeRef>>,
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
    subscription: Option<Type>,
    types: Option<Vec<Type>>,
    directives: Option<Vec<Directive>>,
}

impl Schema {
    pub fn from_url(url: &str) -> Result<Schema, Box<dyn Error>> {
        let client = Client::new();
        let text = client
            .post(url)
            .header("Content-Type", "application/graphql")
            .body(SCHEMA_QUERY)
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
}
