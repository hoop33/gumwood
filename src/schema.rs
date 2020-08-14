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
}

impl fmt::Display for Type {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "{}", name),
            None => write!(f, ""),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Directive {
    name: String,
    description: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Schema {
    #[serde(alias = "queryType")]
    pub query_type: Option<Type>,
    #[serde(alias = "mutationType")]
    pub mutation_type: Option<Type>,
    subscription: Option<Type>,
    types: Vec<Type>,
    directives: Vec<Directive>,
}

impl Schema {
    pub fn from_url(url: &str) -> Result<Schema, Box<dyn Error>> {
        let client = Client::new();
        let json = client
            .post(url)
            .header("Content-Type", "application/graphql")
            .body(SCHEMA_QUERY)
            .send()?
            .json::<Value>()?;
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
