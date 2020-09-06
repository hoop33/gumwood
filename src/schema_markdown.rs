use super::markdown::*;
use super::schema::{Field, Input, Schema, Type};
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

        contents.insert("queries".to_string(), queries_to_markdown(schema));
        contents.insert("mutations".to_string(), mutations_to_markdown(schema));
        contents.insert(
            "subscriptions".to_string(),
            subscriptions_to_markdown(schema),
        );
        contents.insert(
            "inputs".to_string(),
            types_to_markdown(schema, "INPUT_OBJECT"),
        );
        contents.insert("objects".to_string(), types_to_markdown(schema, "OBJECT"));

        contents
    }
}

fn queries_to_markdown(schema: &Schema) -> String {
    match schema
        .get_query_name()
        .and_then(|query_name| schema.get_type(&query_name))
    {
        Some(query_type) => {
            let mut s = String::new();

            match &query_type.name {
                Some(name) => s.push_str(&to_header(1, &name)),
                None => {}
            }

            match &query_type.description {
                Some(description) => s.push_str(&to_description(&description)),
                None => {}
            }

            match &query_type.fields {
                Some(fields) => {
                    for field in fields.iter() {
                        s.push_str(&field_to_markdown(field));
                    }
                }
                None => {}
            }

            s
        }
        None => "".to_string(),
    }
}

fn mutations_to_markdown(schema: &Schema) -> String {
    match schema
        .get_mutation_name()
        .and_then(|mutation_name| schema.get_type(&mutation_name))
    {
        Some(mutation_type) => {
            let mut s = String::new();

            match &mutation_type.name {
                Some(name) => s.push_str(&to_header(1, &name)),
                None => {}
            }

            match &mutation_type.description {
                Some(description) => s.push_str(&to_description(&description)),
                None => {}
            }

            match &mutation_type.inputs {
                Some(inputs) => {
                    for input in inputs {
                        s.push_str(&input_to_markdown(input));
                    }
                }
                None => {}
            }

            match &mutation_type.fields {
                Some(fields) => {
                    for field in fields.iter() {
                        s.push_str(&field_to_markdown(field));
                    }
                }
                None => {}
            }

            s
        }
        None => "".to_string(),
    }
}

fn subscriptions_to_markdown(schema: &Schema) -> String {
    match schema
        .get_subscription_name()
        .and_then(|subscription_name| schema.get_type(&subscription_name))
    {
        Some(subscription_type) => {
            let mut s = String::new();

            match &subscription_type.name {
                Some(name) => s.push_str(&to_header(1, &name)),
                None => {}
            }

            match &subscription_type.description {
                Some(description) => s.push_str(&to_description(&description)),
                None => {}
            }

            match &subscription_type.inputs {
                Some(inputs) => {
                    for input in inputs {
                        s.push_str(&input_to_markdown(input));
                    }
                }
                None => {}
            }

            match &subscription_type.fields {
                Some(fields) => {
                    for field in fields.iter() {
                        s.push_str(&field_to_markdown(field));
                    }
                }
                None => {}
            }

            s
        }
        None => "".to_string(),
    }
}

fn types_to_markdown(schema: &Schema, kind: &str) -> String {
    let mut s = String::new();

    let types = schema.get_types_of_kind(kind);
    for typ in types.iter() {
        s.push_str(&type_to_markdown(typ));
    }

    s
}

fn type_to_markdown(typ: &Type) -> String {
    let mut s = String::new();

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

    match &typ.inputs {
        Some(inputs) => {
            let mut sorted = inputs.to_vec();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            s.push_str(&inputs_to_markdown_table(&sorted));
        }
        None => {}
    }

    s.push_str("\n");

    s
}

fn inputs_to_markdown_table(inputs: &Vec<Input>) -> String {
    let mut s = String::new();

    s.push_str(&to_header(2, "Inputs"));
    let headers = vec!["Name", "Type", "Description", "Default Value"];
    s.push_str(&to_table_row(&headers));
    s.push_str(&to_table_separator(headers.len()));

    for input in inputs.iter() {
        s.push_str(&input_to_markdown_table_row(input));
    }

    s
}

fn input_to_markdown_table_row(input: &Input) -> String {
    let name = match &input.name {
        Some(name) => name.trim(),
        None => "(unknown)",
    };
    let type_name = match input.input_type.as_ref() {
        Some(typ) => typ.to_string(),
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

    to_table_row(&vec![&name, &type_name, &description, &default_value])
}

fn input_to_markdown(input: &Input) -> String {
    let mut s = String::new();

    match &input.name {
        Some(name) => s.push_str(&to_header(1, &name)),
        None => {}
    }

    match &input.description {
        Some(description) => s.push_str(&to_description(&description)),
        None => {}
    }

    s
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
        Some(typ) => match &typ.name {
            Some(name) => s.push_str(&to_label("Type", &name)),
            None => {}
        },
        None => {}
    }

    match &field.args {
        Some(args) => {
            if args.len() > 0 {
                s.push_str(&to_header(3, "Arguments"));
                s.push_str(&to_table_row(&vec!["Name", "Type", "Kind", "Description"]));
                s.push_str(&to_table_separator(4));
                for arg in args {
                    let name = match &arg.name {
                        Some(name) => name.trim(),
                        None => "(unknown)",
                    };
                    let type_name = match arg.input_type.as_ref().and_then(|typ| typ.name.as_ref())
                    {
                        Some(type_name) => type_name.clone(),
                        None => "".to_string(),
                    };
                    let kind = match arg.input_type.as_ref().and_then(|typ| typ.kind.as_ref()) {
                        Some(kind) => kind.clone(),
                        None => "".to_string(),
                    };
                    let description = match &arg.description {
                        Some(description) => description.trim().replace("\n", ""),
                        None => "".to_string(),
                    };
                    s.push_str(&to_table_row(&vec![&name, &type_name, &kind, &description]));
                }
                s.push_str("\n");
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
        assert_eq!(5, map.len());
        assert_eq!("".to_string(), map["queries"]);
        assert_eq!("".to_string(), map["mutations"]);
        assert_eq!("".to_string(), map["subscriptions"]);
        assert_eq!("".to_string(), map["inputs"]);
        assert_eq!("".to_string(), map["objects"]);
    }

    #[test]
    fn test_queries_to_markdown_should_return_empty_when_none() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!("".to_string(), queries_to_markdown(schema));
    }

    #[test]
    fn test_queries_to_markdown_should_return_empty_when_some_and_no_members() {
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
        assert_eq!("".to_string(), queries_to_markdown(schema));
    }

    #[test]
    fn test_queries_to_markdown_should_return_markdown_when_some() {
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
            queries_to_markdown(schema)
        );
    }

    #[test]
    fn test_mutations_to_markdown_should_return_empty_when_none() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!("".to_string(), mutations_to_markdown(schema));
    }

    #[test]
    fn test_mutations_to_markdown_should_return_empty_when_some_and_no_members() {
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
        assert_eq!("".to_string(), mutations_to_markdown(schema));
    }

    #[test]
    fn test_mutations_to_markdown_should_return_markdown_when_some() {
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
            mutations_to_markdown(schema)
        );
    }

    #[test]
    fn test_subscriptions_to_markdown_should_return_empty_when_none() {
        let schema = &Schema {
            query_type: None,
            mutation_type: None,
            subscription_type: None,
            types: None,
            directives: None,
        };
        assert_eq!("".to_string(), subscriptions_to_markdown(schema));
    }

    #[test]
    fn test_subscriptions_to_markdown_should_return_empty_when_some_and_no_members() {
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
        assert_eq!("".to_string(), subscriptions_to_markdown(schema));
    }

    #[test]
    fn test_subscriptions_to_markdown_should_return_markdown_when_some() {
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
            subscriptions_to_markdown(schema)
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
            r#"# Player

> This is a player

## id

> The ID


"#
            .to_string(),
            type_to_markdown(typ)
        );
    }

    #[test]
    fn test_input_to_markdown_should_return_markdown() {
        let input = &Input {
            name: Some("PlayerInput".to_string()),
            description: Some("Input for defining a player".to_string()),
            input_type: None,
            default_value: None,
        };
        assert_eq!(
            r#"# PlayerInput

> Input for defining a player

"#
            .to_string(),
            input_to_markdown(input)
        );
    }
}
