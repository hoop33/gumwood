use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{boxed::Box, error::Error, fmt, fs, path::PathBuf};

const TYPE_LEVELS: u32 = 7;

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
    #[serde(alias = "inputFields")]
    pub inputs: Option<Vec<Input>>,
    pub interfaces: Option<Vec<TypeRef>>,
    #[serde(alias = "enumValues")]
    pub enums: Option<Vec<Enum>>,
    #[serde(alias = "possibleTypes")]
    pub possible_types: Option<Vec<TypeRef>>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
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

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Input {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "type")]
    pub input_type: Option<TypeRef>,
    #[serde(alias = "defaultValue")]
    pub default_value: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Enum {
    pub name: Option<String>,
    pub description: Option<String>,
    #[serde(alias = "isDeprecated")]
    pub is_deprecated: Option<bool>,
    #[serde(alias = "deprecationReason")]
    pub deprecation_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
pub struct TypeRef {
    pub name: Option<String>,
    pub kind: Option<String>,
    #[serde(alias = "ofType")]
    pub of_type: Option<Box<TypeRef>>,
}

impl TypeRef {
    pub fn is_required(&self) -> bool {
        self.kind.is_some() && self.kind.as_ref().unwrap() == "NON_NULL"
    }

    pub fn is_list(&self) -> bool {
        self.kind.is_some() && self.kind.as_ref().unwrap() == "LIST"
    }

    pub fn get_actual_name(&self) -> String {
        self.recurse_actual_name(TYPE_LEVELS)
    }

    fn recurse_actual_name(&self, level: u32) -> String {
        if level == 0 {
            return "".to_string();
        }

        match &self.name {
            Some(name) => name.to_string(),
            None => match &self.of_type {
                Some(typ) => typ.recurse_actual_name(level - 1),
                None => "".to_string(),
            },
        }
    }

    pub fn get_decorated_name(&self) -> String {
        self.recurse_decorated_name(TYPE_LEVELS)
    }

    fn recurse_decorated_name(&self, level: u32) -> String {
        if level == 0 {
            return "".to_string();
        }

        let mut s = String::new();

        let name = match &self.name {
            Some(name) => name.clone(),
            None => match &self.of_type {
                Some(typ) => typ.recurse_decorated_name(level - 1),
                None => "".to_string(),
            },
        };

        s.push_str(&name);

        if self.is_required() {
            s.push_str("!");
        }

        if self.is_list() {
            s.insert_str(0, "[");
            s.push_str("]");
        }

        s
    }

    pub fn get_actual_kind(&self) -> String {
        self.recurse_actual_kind(TYPE_LEVELS)
    }

    fn recurse_actual_kind(&self, level: u32) -> String {
        if level == 0 {
            return "".to_string();
        }

        // When we encounter ofType: null, we have the kind
        match &self.of_type {
            Some(typ) => typ.recurse_actual_kind(level - 1),
            None => match &self.kind {
                Some(kind) => kind.to_string(),
                None => "".to_string(),
            },
        }
    }
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
    pub fn from_url(url: &str, headers: &[String]) -> Result<Schema, Box<dyn Error>> {
        let client = Client::new();
        let mut post = client.post(url);
        for header in headers {
            let split: Vec<&str> = header.split(':').collect();
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

    pub fn from_json(file: &PathBuf) -> Result<Schema, Box<dyn Error>> {
        let contents = fs::read_to_string(file)?;
        Schema::from_str(&contents)
    }

    pub fn from_schema(_file: &PathBuf) -> Result<Schema, Box<dyn Error>> {
        Err(Box::new(SchemaError::new("not yet implemented")))
    }

    pub fn from_str(text: &str) -> Result<Schema, Box<dyn Error>> {
        match serde_json::from_str(&text)? {
            Value::Object(map) => match map.get("data") {
                Some(data) => match data.get("__schema") {
                    Some(schema) => {
                        let s: Schema = serde_json::from_str(&schema.to_string())?;
                        Ok(s)
                    }
                    None => Err(Box::new(SchemaError::new("schema not in response"))),
                },
                None => Err(Box::new(SchemaError::new("data not in response"))),
            },
            _ => {
                // I don't think this is reachable; as far as I can tell,
                // serde_json::from_str() fails if text is not a JSON object.
                // You can't pass it an array, for example. So if line 14 passes,
                // we're already guaranteed to have an object.
                Err(Box::new(SchemaError::new("response format not an object")))
            }
        }
    }

    pub fn get_query_name(&self) -> Option<String> {
        Schema::get_type_name(&self.query_type)
    }

    pub fn get_mutation_name(&self) -> Option<String> {
        Schema::get_type_name(&self.mutation_type)
    }

    pub fn get_subscription_name(&self) -> Option<String> {
        Schema::get_type_name(&self.subscription_type)
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
                None
            }
            None => None,
        }
    }

    pub fn get_types_of_kind(&self, kind: &str) -> Vec<&Type> {
        let mut vec = Vec::new();

        match &self.types {
            Some(types) => {
                for typ in types.iter() {
                    match &typ.kind {
                        Some(k) => {
                            if k == kind {
                                vec.push(typ);
                            }
                        }
                        None => {}
                    }
                }
            }
            None => {}
        }

        vec
    }

    fn get_type_name(typ: &Option<Type>) -> Option<String> {
        typ.as_ref().and_then(|typ| typ.name.clone())
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
    fn from_str_should_pass_when_empty_schema() {
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
    fn from_str_should_fail_when_not_json() {
        let response = "test";
        match Schema::from_str(response) {
            Ok(_) => assert!(false, "plain text should fail"),
            Err(err) => assert_eq!("expected ident at line 1 column 2", err.to_string()),
        }
    }

    #[test]
    fn from_str_should_fail_when_no_data() {
        let response = r#"{
        }"#;
        match Schema::from_str(&response) {
            Ok(_) => assert!(false, "schema should have data"),
            Err(err) => assert_eq!("data not in response", err.to_string()),
        }
    }

    #[test]
    fn from_str_should_fail_when_no_schema() {
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
    fn from_str_should_have_no_query_type_when_none() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_query_type_when_some() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_query_type_name_when_present() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_some_query_name_when_present() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_query_name_when_absent() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_query_name_when_name_absent() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_no_mutation_type_when_none() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_mutation_type_when_some() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_mutation_type_name_when_present() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_some_mutation_name_when_present() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_mutation_name_when_absent() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_mutation_name_when_name_absent() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_no_subscription_type_when_none() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_subscription_type_when_some() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_have_subscription_type_name_when_present() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_some_subscription_name_when_present() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_subscription_name_when_absent() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_subscription_name_when_name_absent() -> Result<(), Box<dyn Error>>
    {
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
    fn from_str_should_return_none_when_no_types() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_when_type_has_no_name() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_none_when_no_type_of_name() -> Result<(), Box<dyn Error>> {
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
    fn from_str_should_return_some_when_type_of_name() -> Result<(), Box<dyn Error>> {
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

    #[test]
    fn typeref_is_required_should_return_false_when_kind_is_none() {
        let tr = TypeRef {
            name: None,
            kind: None,
            of_type: None,
        };
        assert!(!tr.is_required());
    }

    #[test]
    fn typeref_is_required_should_return_false_when_kind_is_not_non_null() {
        let tr = TypeRef {
            name: None,
            kind: Some("foo".to_string()),
            of_type: None,
        };
        assert!(!tr.is_required());
    }

    #[test]
    fn typeref_is_required_should_return_true_when_kind_is_non_null() {
        let tr = TypeRef {
            name: None,
            kind: Some("NON_NULL".to_string()),
            of_type: None,
        };
        assert!(tr.is_required());
    }

    #[test]
    fn typeref_is_list_should_return_false_when_kind_is_none() {
        let tr = TypeRef {
            name: None,
            kind: None,
            of_type: None,
        };
        assert!(!tr.is_list());
    }

    #[test]
    fn typeref_is_list_should_return_false_when_kind_is_not_non_null() {
        let tr = TypeRef {
            name: None,
            kind: Some("foo".to_string()),
            of_type: None,
        };
        assert!(!tr.is_list());
    }

    #[test]
    fn typeref_is_list_should_return_true_when_kind_is_non_null() {
        let tr = TypeRef {
            name: None,
            kind: Some("LIST".to_string()),
            of_type: None,
        };
        assert!(tr.is_list());
    }

    #[test]
    fn typeref_decorated_name_should_return_empty_when_none() {
        let tr = TypeRef {
            name: None,
            kind: None,
            of_type: None,
        };
        assert_eq!("", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_when_not_required() {
        let tr = TypeRef {
            name: Some("myName".to_string()),
            kind: None,
            of_type: None,
        };
        assert_eq!("myName", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_with_exclamation_when_required() {
        let tr = TypeRef {
            name: Some("myName".to_string()),
            kind: Some("NON_NULL".to_string()),
            of_type: None,
        };
        assert_eq!("myName!", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_with_brackets_when_list() {
        let tr = TypeRef {
            name: Some("myName".to_string()),
            kind: Some("LIST".to_string()),
            of_type: None,
        };
        assert_eq!("[myName]", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_with_brackets_and_exclamation_when_list_and_required(
    ) {
        let tr = TypeRef {
            name: None,
            kind: Some("LIST".to_string()),
            of_type: Some(Box::new(TypeRef {
                kind: Some("NON_NULL".to_string()),
                name: Some("myName".to_string()),
                of_type: None,
            })),
        };
        assert_eq!("[myName!]", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_with_brackets_and_exclamation_outside_when_list_and_required(
    ) {
        let tr = TypeRef {
            name: None,
            kind: Some("NON_NULL".to_string()),
            of_type: Some(Box::new(TypeRef {
                kind: Some("LIST".to_string()),
                name: Some("myName".to_string()),
                of_type: None,
            })),
        };
        assert_eq!("[myName]!", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_with_brackets_and_two_exclamation_when_list_and_required(
    ) {
        let tr = TypeRef {
            name: None,
            kind: Some("NON_NULL".to_string()),
            of_type: Some(Box::new(TypeRef {
                kind: Some("LIST".to_string()),
                name: None,
                of_type: Some(Box::new(TypeRef {
                    kind: Some("NON_NULL".to_string()),
                    name: Some("myName".to_string()),
                    of_type: None,
                })),
            })),
        };
        assert_eq!("[myName!]!", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_return_name_with_brackets_when_not_scalar() {
        let tr = TypeRef {
            name: None,
            kind: Some("LIST".to_string()),
            of_type: Some(Box::new(TypeRef {
                kind: Some("INPUT_OBJECT".to_string()),
                name: Some("MyInputObject".to_string()),
                of_type: None,
            })),
        };
        assert_eq!("[MyInputObject]", tr.get_decorated_name());
    }

    #[test]
    fn typeref_decorated_name_should_short_circuit_when_nested_too_deep() {
        let tr = TypeRef {
            name: None,
            kind: None,
            of_type: Some(Box::new(TypeRef {
                name: None,
                kind: None,
                of_type: Some(Box::new(TypeRef {
                    name: None,
                    kind: None,
                    of_type: Some(Box::new(TypeRef {
                        name: None,
                        kind: None,
                        of_type: Some(Box::new(TypeRef {
                            name: None,
                            kind: None,
                            of_type: Some(Box::new(TypeRef {
                                name: None,
                                kind: None,
                                of_type: Some(Box::new(TypeRef {
                                    name: None,
                                    kind: None,
                                    of_type: Some(Box::new(TypeRef {
                                        name: None,
                                        kind: None,
                                        of_type: Some(Box::new(TypeRef {
                                            name: None,
                                            kind: None,
                                            of_type: None,
                                        })),
                                    })),
                                })),
                            })),
                        })),
                    })),
                })),
            })),
        };
        assert_eq!("", tr.get_decorated_name());
    }

    #[test]
    fn typeref_actual_kind_should_return_empty_when_none() {
        let tr = TypeRef {
            name: None,
            kind: None,
            of_type: None,
        };
        assert_eq!("", tr.get_actual_kind());
    }

    #[test]
    fn typeref_actual_kind_should_return_kind_when_of_type_is_none() {
        let tr = TypeRef {
            name: None,
            kind: Some("SCALAR".to_string()),
            of_type: None,
        };
        assert_eq!("SCALAR", tr.get_actual_kind());
    }

    #[test]
    fn typeref_actual_kind_should_return_nested_kind_when_of_type_is_some() {
        let tr = TypeRef {
            name: None,
            kind: Some("NON_NULL".to_string()),
            of_type: Some(Box::new(TypeRef {
                name: None,
                kind: Some("OBJECT".to_string()),
                of_type: None,
            })),
        };
        assert_eq!("OBJECT", tr.get_actual_kind());
    }

    #[test]
    fn typeref_actual_kind_should_return_third_nested_kind_when_of_type_is_some() {
        let tr = TypeRef {
            name: None,
            kind: Some("NON_NULL".to_string()),
            of_type: Some(Box::new(TypeRef {
                name: None,
                kind: Some("LIST".to_string()),
                of_type: Some(Box::new(TypeRef {
                    name: None,
                    kind: Some("OBJECT".to_string()),
                    of_type: None,
                })),
            })),
        };
        assert_eq!("OBJECT", tr.get_actual_kind());
    }

    #[test]
    fn typeref_actual_kind_should_short_circuit_when_nested_too_deep() {
        let tr = TypeRef {
            name: None,
            kind: None,
            of_type: Some(Box::new(TypeRef {
                name: None,
                kind: None,
                of_type: Some(Box::new(TypeRef {
                    name: None,
                    kind: None,
                    of_type: Some(Box::new(TypeRef {
                        name: None,
                        kind: None,
                        of_type: Some(Box::new(TypeRef {
                            name: None,
                            kind: None,
                            of_type: Some(Box::new(TypeRef {
                                name: None,
                                kind: None,
                                of_type: Some(Box::new(TypeRef {
                                    name: None,
                                    kind: None,
                                    of_type: Some(Box::new(TypeRef {
                                        name: None,
                                        kind: None,
                                        of_type: Some(Box::new(TypeRef {
                                            name: None,
                                            kind: None,
                                            of_type: None,
                                        })),
                                    })),
                                })),
                            })),
                        })),
                    })),
                })),
            })),
        };
        assert_eq!("", tr.get_actual_kind());
    }

    #[test]
    fn get_types_of_kind_should_return_only_types_of_kind() {
        let response = r#"{
        "data": {
            "__schema": {
                "types": [
                    {
                        "kind": "FOO"
                    },
                    {
                        "kind": "BAR"
                    },
                    {
                        "kind": "FOO"
                    },
                    {
                        "name": "FOO"
                    }
                ]
            }
        }
    }"#;
        let schema = Schema::from_str(&response).unwrap();
        assert_eq!(2, schema.get_types_of_kind("FOO").len());
    }

    #[test]
    fn get_types_of_kind_should_return_empty_when_types_are_none() {
        let response = r#"{
            "data": {
                "__schema": {
                }
            }
        }"#;
        let schema = Schema::from_str(&response).unwrap();
        assert_eq!(0, schema.get_types_of_kind("FOO").len());
    }

    #[test]
    fn get_types_of_kind_should_return_empty_when_no_types_match() {
        let response = r#"{
        "data": {
            "__schema": {
                "types": [
                    {
                        "kind": "FOO"
                    },
                    {
                        "kind": "FOO"
                    },
                    {
                        "name": "FOO"
                    }
                ]
            }
        }
    }"#;
        let schema = Schema::from_str(&response).unwrap();
        assert_eq!(0, schema.get_types_of_kind("BAR").len());
    }
}
