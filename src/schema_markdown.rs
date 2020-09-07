use super::markdown::*;
use super::schema::{Enum, Field, Input, Schema, Type};
use std::collections::HashMap;
use std::error::Error;

#[derive(Debug)]
pub struct Markdown {
    front_matter: Option<String>,
}

impl Markdown {
    pub fn with_front_matter(front_matter: Option<String>) -> Result<Markdown, Box<dyn Error>> {
        Ok(Markdown { front_matter })
    }

    pub fn generate_from_schema(&self, schema: &Schema) -> HashMap<String, String> {
        let mut contents: HashMap<String, String> = HashMap::new();

        contents.insert(
            "queries".to_string(),
            schema_type_to_markdown(schema, schema.get_query_name()),
        );
        contents.insert(
            "mutations".to_string(),
            schema_type_to_markdown(schema, schema.get_mutation_name()),
        );
        contents.insert(
            "subscriptions".to_string(),
            schema_type_to_markdown(schema, schema.get_subscription_name()),
        );
        contents.insert(
            "inputs".to_string(),
            types_to_markdown(schema, "Inputs", "INPUT_OBJECT"),
        );
        contents.insert(
            "objects".to_string(),
            types_to_markdown(schema, "Objects", "OBJECT"),
        );
        contents.insert(
            "enums".to_string(),
            types_to_markdown(schema, "Enums", "ENUM"),
        );
        contents.insert(
            "interfaces".to_string(),
            types_to_markdown(schema, "Interfaces", "INTERFACE"),
        );
        contents.insert(
            "unions".to_string(),
            types_to_markdown(schema, "Unions", "UNION"),
        );
        contents.insert(
            "scalars".to_string(),
            types_to_markdown(schema, "Scalars", "SCALAR"),
        );

        contents
    }
}

fn schema_type_to_markdown(schema: &Schema, type_name: Option<String>) -> String {
    let mut s = String::new();

    if let Some(typ) = type_name.and_then(|name| schema.get_type(&name)) {
        match &typ.name {
            Some(name) => s.push_str(&to_header(1, &name)),
            None => {}
        }

        match &typ.description {
            Some(description) => s.push_str(&to_description(&description)),
            None => {}
        }

        match &typ.fields {
            Some(fields) => {
                for field in fields.iter() {
                    s.push_str(&field_to_markdown(field));
                }
            }
            None => {}
        }
    }

    s
}

fn types_to_markdown(schema: &Schema, title: &str, kind: &str) -> String {
    let mut s = String::new();

    let mut types = schema.get_types_of_kind(kind);

    if !types.is_empty() {
        s.push_str(&to_header(1, title));

        types.sort_by(|a, b| a.name.cmp(&b.name));

        for typ in types.iter() {
            s.push_str(&type_to_markdown(typ));
        }
    }

    s
}

fn type_to_markdown(typ: &Type) -> String {
    let mut s = String::new();

    match &typ.name {
        Some(name) => s.push_str(&to_header(2, &name)),
        None => {}
    }

    match &typ.description {
        Some(description) => s.push_str(&to_description(&description)),
        None => {}
    }

    match &typ.fields {
        Some(fields) => {
            s.push_str(&to_header(3, "Fields"));
            let mut sorted = fields.to_vec();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            s.push_str(&fields_to_markdown_table(&sorted));
        }
        None => {}
    }

    match &typ.inputs {
        Some(inputs) => {
            s.push_str(&to_header(3, "Inputs"));
            let mut sorted = inputs.to_vec();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            s.push_str(&inputs_to_markdown_table(&sorted));
        }
        None => {}
    }

    match &typ.enums {
        Some(enums) => {
            s.push_str(&to_header(3, "Values"));
            let mut sorted = enums.to_vec();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            s.push_str(&enums_to_markdown_table(&sorted));
        }
        None => {}
    }

    match &typ.possible_types {
        Some(possible_types) => {
            s.push_str(&to_header(3, "Implemented by"));
            let mut names: Vec<&str> = possible_types
                .iter()
                .map(|typ| match &typ.name {
                    Some(name) => name,
                    None => "",
                })
                .collect();
            names.sort();
            s.push_str(&to_list(&names));
        }
        None => {}
    }

    s
}

fn fields_to_markdown_table(fields: &[Field]) -> String {
    let mut s = String::new();

    let headers = vec!["Name", "Type", "Description"];
    s.push_str(&to_table_row(&headers));
    s.push_str(&to_table_separator(headers.len()));

    for field in fields.iter() {
        s.push_str(&field_to_markdown_table_row(field));
    }

    s.push_str("\n");

    s
}

fn inputs_to_markdown_table(inputs: &[Input]) -> String {
    let mut s = String::new();

    let headers = vec!["Name", "Type", "Description", "Default Value"];
    s.push_str(&to_table_row(&headers));
    s.push_str(&to_table_separator(headers.len()));

    for input in inputs.iter() {
        s.push_str(&input_to_markdown_table_row(input));
    }

    s.push_str("\n");

    s
}

fn enums_to_markdown_table(enums: &[Enum]) -> String {
    let mut s = String::new();

    let headers = vec!["Name", "Description", "Deprecated?"];
    s.push_str(&to_table_row(&headers));
    s.push_str(&to_table_separator(headers.len()));

    for enm in enums.iter() {
        s.push_str(&enum_to_markdown_table_row(enm));
    }

    s.push_str("\n");

    s
}

fn field_to_markdown_table_row(field: &Field) -> String {
    let name = match &field.name {
        Some(name) => name.trim(),
        None => "(unknown)",
    };
    let type_name = match field.field_type.as_ref() {
        Some(typ) => typ.decorated_name(),
        None => "".to_string(),
    };
    let description = match &field.description {
        Some(description) => description.trim().replace("\n", ""),
        None => "".to_string(),
    };

    to_table_row(&[&name, &type_name, &description])
}

fn input_to_markdown_table_row(input: &Input) -> String {
    let name = match &input.name {
        Some(name) => name.trim(),
        None => "(unknown)",
    };
    let type_name = match input.input_type.as_ref() {
        Some(typ) => typ.decorated_name(),
        None => "".to_string(),
    };
    let description = match &input.description {
        Some(description) => description.trim().replace("\n", ""),
        None => "".to_string(),
    };
    let default_value = match &input.default_value {
        Some(default_value) => default_value.trim().replace("\n", ""),
        None => "".to_string(),
    };

    to_table_row(&[&name, &type_name, &description, &default_value])
}

fn enum_to_markdown_table_row(enm: &Enum) -> String {
    let name = match &enm.name {
        Some(name) => name.trim(),
        None => "(unknown)",
    };
    let description = match &enm.description {
        Some(description) => description.trim().replace("\n", ""),
        None => "".to_string(),
    };
    let is_deprecated = match &enm.is_deprecated {
        Some(is_deprecated) => *is_deprecated,
        None => false,
    };
    let deprecation_reason = match &enm.deprecation_reason {
        Some(deprecation_reason) => deprecation_reason,
        None => "",
    };

    let dr = if is_deprecated {
        deprecation_reason
    } else {
        "no"
    };

    to_table_row(&[&name, &description, &dr])
}

fn field_to_markdown(field: &Field) -> String {
    let mut s = String::new();

    match &field.name {
        Some(name) => s.push_str(&to_header(2, &name)),
        None => {}
    }

    match &field.is_deprecated {
        Some(deprecated) => {
            if *deprecated {
                s.push_str(&to_notice("Deprecated"));
            }
        }
        None => {}
    }

    match &field.description {
        Some(description) => s.push_str(&to_description(&description)),
        None => {}
    }

    match &field.field_type {
        Some(typ) => s.push_str(&to_label("Type", &typ.decorated_name())),
        None => {}
    }

    match &field.args {
        Some(args) => {
            if !args.is_empty() {
                s.push_str(&to_header(3, "Arguments"));
                let mut sorted = args.to_vec();
                sorted.sort_by(|a, b| a.name.cmp(&b.name));
                s.push_str(&inputs_to_markdown_table(&sorted));
            }
        }
        None => {}
    }

    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_front_matter_should_return_ok_when_none() {
        assert!(Markdown::with_front_matter(None).is_ok());
    }

    #[test]
    fn test_with_front_matter_should_return_ok_when_some() {
        assert!(Markdown::with_front_matter(Some("fm:foo".to_string())).is_ok());
    }

    #[test]
    fn test_generate_from_schema_should_return_empty_when_empty_schema() {
        let markdown = Markdown::with_front_matter(None).unwrap();
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        let map = markdown.generate_from_schema(schema);
        assert_eq!(9, map.len());
        assert_eq!("".to_string(), map["queries"]);
        assert_eq!("".to_string(), map["mutations"]);
        assert_eq!("".to_string(), map["subscriptions"]);
        assert_eq!("".to_string(), map["inputs"]);
        assert_eq!("".to_string(), map["objects"]);
        assert_eq!("".to_string(), map["enums"]);
        assert_eq!("".to_string(), map["interfaces"]);
        assert_eq!("".to_string(), map["unions"]);
        assert_eq!("".to_string(), map["scalars"]);
    }

    #[test]
    fn test_schema_type_to_markdown_for_query_should_return_empty_when_none() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!(
            "".to_string(),
            schema_type_to_markdown(schema, schema.get_query_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_query_should_return_empty_when_some_and_no_members() {
        let schema = &Schema {
            query_type: Some(Type {
                name: None,
                kind: None,
                description: None,
                fields: None,
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }),
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!(
            "".to_string(),
            schema_type_to_markdown(schema, schema.get_query_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_query_should_return_markdown_when_some() {
        let schema = &Schema {
            query_type: Some(Type {
                name: Some("Query".to_string()),
                kind: None,
                description: None,
                fields: None,
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }),
            mutation_type: None,
            subscription_type: None,
            types: Some(vec![Type {
                name: Some("Query".to_string()),
                kind: None,
                description: Some("The root query".to_string()),
                fields: Some(vec![Field {
                    name: Some("players".to_string()),
                    description: Some("get the players".to_string()),
                    args: None,
                    field_type: None,
                    is_deprecated: None,
                    deprecation_reason: None,
                }]),
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }]),
            directives: None,
        };
        assert_eq!(
            r#"# Query

> The root query

## players

> get the players

"#
            .to_string(),
            schema_type_to_markdown(schema, schema.get_query_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_mutation_should_return_empty_when_none() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!(
            "".to_string(),
            schema_type_to_markdown(schema, schema.get_mutation_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_mutation_should_return_empty_when_some_and_no_members() {
        let schema = &Schema {
            query_type: None,
            mutation_type: Some(Type {
                name: None,
                kind: None,
                description: None,
                fields: None,
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }),
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!(
            "".to_string(),
            schema_type_to_markdown(schema, schema.get_mutation_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_mutation_should_return_markdown_when_some() {
        let schema = &Schema {
            query_type: None,
            mutation_type: Some(Type {
                name: Some("Mutation".to_string()),
                kind: None,
                description: None,
                fields: None,
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }),
            subscription_type: None,
            types: Some(vec![Type {
                name: Some("Mutation".to_string()),
                kind: None,
                description: Some("The root mutation".to_string()),
                fields: Some(vec![Field {
                    name: Some("addPlayer".to_string()),
                    description: Some("add a player".to_string()),
                    args: None,
                    field_type: None,
                    is_deprecated: None,
                    deprecation_reason: None,
                }]),
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }]),
            directives: None,
        };
        assert_eq!(
            r#"# Mutation

> The root mutation

## addPlayer

> add a player

"#
            .to_string(),
            schema_type_to_markdown(schema, schema.get_mutation_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_subscription_should_return_empty_when_none() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!(
            "".to_string(),
            schema_type_to_markdown(schema, schema.get_subscription_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_subscription_should_return_empty_when_some_and_no_members()
    {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: Some(Type {
                name: None,
                kind: None,
                description: None,
                fields: None,
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }),
            types: None,
            directives: None,
        };
        assert_eq!(
            "".to_string(),
            schema_type_to_markdown(schema, schema.get_subscription_name())
        );
    }

    #[test]
    fn test_schema_type_to_markdown_for_subscription_should_return_markdown_when_some() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: Some(Type {
                name: Some("Subscription".to_string()),
                kind: None,
                description: None,
                fields: None,
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }),
            types: Some(vec![Type {
                name: Some("Subscription".to_string()),
                kind: None,
                description: Some("The root subscription".to_string()),
                fields: Some(vec![Field {
                    name: Some("subscribePlayers".to_string()),
                    description: Some("subscribe to players".to_string()),
                    args: None,
                    field_type: None,
                    is_deprecated: None,
                    deprecation_reason: None,
                }]),
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }]),
            directives: None,
        };
        assert_eq!(
            r#"# Subscription

> The root subscription

## subscribePlayers

> subscribe to players

"#
            .to_string(),
            schema_type_to_markdown(schema, schema.get_subscription_name())
        );
    }

    #[test]
    fn test_types_to_markdown_should_return_markdown() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: Some(vec![Type {
                name: Some("Player".to_string()),
                kind: Some("OBJECT".to_string()),
                description: Some("A player".to_string()),
                fields: Some(vec![
                    Field {
                        name: Some("firstName".to_string()),
                        description: Some("The player's first name".to_string()),
                        args: None,
                        field_type: None,
                        is_deprecated: None,
                        deprecation_reason: None,
                    },
                    Field {
                        name: Some("lastName".to_string()),
                        description: Some("The player's last name".to_string()),
                        args: None,
                        field_type: None,
                        is_deprecated: None,
                        deprecation_reason: None,
                    },
                ]),
                inputs: None,
                interfaces: None,
                enums: None,
                possible_types: None,
            }]),
            directives: None,
        };
        assert_eq!(
            r#"# Objects

## Player

> A player

### Fields

| Name | Type | Description |
| --- | --- | --- |
| firstName |  | The player's first name |
| lastName |  | The player's last name |

"#
            .to_string(),
            types_to_markdown(schema, "Objects", "OBJECT")
        );
    }

    #[test]
    fn test_type_to_markdown_should_return_markdown() {
        let typ = &Type {
            name: Some("Player".to_string()),
            description: Some("This is a player".to_string()),
            kind: None,
            inputs: None,
            interfaces: None,
            enums: None,
            possible_types: None,
            fields: Some(vec![Field {
                name: Some("id".to_string()),
                description: Some("The ID".to_string()),
                args: None,
                field_type: None,
                is_deprecated: None,
                deprecation_reason: None,
            }]),
        };
        assert_eq!(
            r#"## Player

> This is a player

### Fields

| Name | Type | Description |
| --- | --- | --- |
| id |  | The ID |

"#
            .to_string(),
            type_to_markdown(typ)
        );
    }
}
