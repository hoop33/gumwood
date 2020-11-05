use super::markdown::*;
use super::schema::{Enum, Field, Input, Schema, Type, TypeRef};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::error::Error;
use titlecase::titlecase;

lazy_static! {
    static ref GRAPHQL_TYPES: HashMap<&'static str, &'static str> = {
        let mut map = HashMap::new();
        map.insert("INPUT_OBJECT", "inputs");
        map.insert("OBJECT", "objects");
        map.insert("ENUM", "enums");
        map.insert("INTERFACE", "interfaces");
        map.insert("UNION", "unions");
        map.insert("SCALAR", "scalars");
        map
    };
}

#[derive(Debug)]
pub struct Markdown {
    multiple: bool,
}

impl Markdown {
    pub fn new(multiple: bool) -> Result<Markdown, Box<dyn Error>> {
        Ok(Markdown { multiple })
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

        for (graphql, friendly) in GRAPHQL_TYPES.iter() {
            contents.insert(
                friendly.to_string(),
                types_to_markdown(schema, &titlecase(friendly), graphql),
            );
        }

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
        Some(name) => s.push_str(&to_header(2, &to_named_anchor(name))),
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
            s.push_str(&to_markdown_table(
                vec![
                    "Name".to_string(),
                    "Type".to_string(),
                    "Description".to_string(),
                ],
                &sorted,
            ));
        }
        None => {}
    }

    match &typ.inputs {
        Some(inputs) => {
            s.push_str(&to_header(3, "Inputs"));
            let mut sorted = inputs.to_vec();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            s.push_str(&to_markdown_table(
                vec![
                    "Name".to_string(),
                    "Type".to_string(),
                    "Description".to_string(),
                    "Default Value".to_string(),
                ],
                &sorted,
            ));
        }
        None => {}
    }

    match &typ.enums {
        Some(enums) => {
            s.push_str(&to_header(3, "Values"));
            let mut sorted = enums.to_vec();
            sorted.sort_by(|a, b| a.name.cmp(&b.name));
            s.push_str(&to_markdown_table(
                vec![
                    "Name".to_string(),
                    "Description".to_string(),
                    "Deprecated".to_string(),
                ],
                &sorted,
            ));
        }
        None => {}
    }

    match &typ.possible_types {
        Some(possible_types) => {
            s.push_str(&to_header(3, "Implemented by"));
            let mut names: Vec<String> = possible_types
                .iter()
                .map(|typ| match &typ.name {
                    Some(name) => to_inline_code(name),
                    None => "".to_string(),
                })
                .collect();
            names.sort();
            s.push_str(&to_list(&names));
        }
        None => {}
    }

    s
}

pub trait TableItem {
    fn table_fields(&self) -> Vec<String>;
}

impl TableItem for Field {
    fn table_fields(&self) -> Vec<String> {
        let type_name = match self.field_type.as_ref() {
            Some(typ) => typ.get_decorated_name(),
            None => "".to_string(),
        };
        let link = match self.field_type.as_ref() {
            Some(typ) => get_link_for_type_ref(typ),
            None => "".to_string(),
        };
        vec![
            to_inline_code(&to_safe_string(&self.name)),
            to_link(&to_inline_code(&type_name), &link),
            to_safe_string(&self.description),
        ]
    }
}

impl TableItem for Input {
    fn table_fields(&self) -> Vec<String> {
        let type_name = match self.input_type.as_ref() {
            Some(typ) => typ.get_decorated_name(),
            None => "".to_string(),
        };
        let link = match self.input_type.as_ref() {
            Some(typ) => get_link_for_type_ref(typ),
            None => "".to_string(),
        };
        vec![
            to_inline_code(&to_safe_string(&self.name)),
            to_link(&to_inline_code(&type_name), &link),
            to_safe_string(&self.description),
            to_inline_code(&to_safe_string(&self.default_value)),
        ]
    }
}

impl TableItem for Enum {
    fn table_fields(&self) -> Vec<String> {
        let is_deprecated = match &self.is_deprecated {
            Some(is_deprecated) => *is_deprecated,
            None => false,
        };
        let deprecation_reason = to_safe_string(&self.deprecation_reason);
        let dr = if is_deprecated {
            deprecation_reason
        } else {
            "no".to_string()
        };
        vec![
            to_inline_code(&to_safe_string(&self.name)),
            to_safe_string(&self.description),
            dr,
        ]
    }
}

fn to_safe_string(opt_s: &Option<String>) -> String {
    match opt_s {
        Some(s) => s.trim().replace("\n", ""),
        None => "".to_string(),
    }
}

fn to_markdown_table(headers: Vec<String>, items: &[impl TableItem]) -> String {
    let mut s = String::new();
    s.push_str(&to_table_row(&headers));
    s.push_str(&to_table_separator(headers.len()));

    for item in items.iter() {
        s.push_str(&to_table_row(&item.table_fields()));
    }
    s.push_str("\n");
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
        Some(typ) => s.push_str(&to_label(
            "Type",
            &to_link(
                &to_inline_code(&typ.get_decorated_name()),
                &get_link_for_type_ref(&typ),
            ),
        )),
        None => {}
    }

    match &field.args {
        Some(args) => {
            if !args.is_empty() {
                s.push_str(&to_header(3, "Arguments"));
                let mut sorted = args.to_vec();
                sorted.sort_by(|a, b| a.name.cmp(&b.name));
                s.push_str(&to_markdown_table(
                    vec![
                        "Name".to_string(),
                        "Type".to_string(),
                        "Description".to_string(),
                        "Default Value".to_string(),
                    ],
                    &sorted,
                ));
            }
        }
        None => {}
    }

    s
}

fn get_link_for_type_ref(type_ref: &TypeRef) -> String {
    let kind = type_ref.get_actual_kind();
    let link_to: &str = match GRAPHQL_TYPES.get::<str>(&kind) {
        Some(friendly) => friendly,
        None => "",
    };
    format!(
        "{}.md#{}",
        link_to,
        type_ref.get_actual_name().to_lowercase()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::TypeRef;

    #[test]
    fn markdown_new_should_return_ok() {
        assert!(Markdown::new(false).is_ok());
    }

    #[test]
    fn generate_from_schema_should_return_empty_when_empty_schema() {
        let markdown = Markdown::new(false).unwrap();
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
    fn schema_type_to_markdown_for_query_should_return_empty_when_none() {
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
    fn schema_type_to_markdown_for_query_should_return_empty_when_some_and_no_members() {
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
    fn schema_type_to_markdown_for_query_should_return_markdown_when_some() {
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
    fn schema_type_to_markdown_for_mutation_should_return_empty_when_none() {
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
    fn schema_type_to_markdown_for_mutation_should_return_empty_when_some_and_no_members() {
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
    fn schema_type_to_markdown_for_mutation_should_return_markdown_when_some() {
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
    fn schema_type_to_markdown_for_subscription_should_return_empty_when_none() {
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
    fn schema_type_to_markdown_for_subscription_should_return_empty_when_some_and_no_members() {
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
    fn schema_type_to_markdown_for_subscription_should_return_markdown_when_some() {
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
    fn types_to_markdown_should_return_markdown() {
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

## <a name="player"></a>Player

> A player

### Fields

| Name | Type | Description |
| --- | --- | --- |
| `firstName` |  | The player's first name |
| `lastName` |  | The player's last name |

"#
            .to_string(),
            types_to_markdown(schema, "Objects", "OBJECT")
        );
    }

    #[test]
    fn type_to_markdown_should_return_markdown() {
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
            r#"## <a name="player"></a>Player

> This is a player

### Fields

| Name | Type | Description |
| --- | --- | --- |
| `id` |  | The ID |

"#
            .to_string(),
            type_to_markdown(typ)
        );
    }

    #[test]
    fn to_safe_string_should_return_string_when_some() {
        assert_eq!(
            "hello".to_string(),
            to_safe_string(&Some("hello".to_string()))
        );
    }

    #[test]
    fn to_safe_string_should_return_empty_string_when_none() {
        assert_eq!("".to_string(), to_safe_string(&None));
    }

    #[test]
    fn input_table_fields_should_return_table_fields_when_some() {
        let input = Input {
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            input_type: Some(TypeRef {
                name: None,
                kind: Some("NON_NULL".to_string()),
                of_type: Some(Box::new(TypeRef {
                    name: Some("ID".to_string()),
                    kind: Some("SCALAR".to_string()),
                    of_type: None,
                })),
            }),
            default_value: Some("default".to_string()),
        };
        let fields = input.table_fields();
        assert_eq!(4, fields.len());
        assert_eq!("`name`".to_string(), fields[0]);
        assert_eq!("[`ID!`](scalars.md#id)".to_string(), fields[1]);
        assert_eq!("description".to_string(), fields[2]);
        assert_eq!("`default`".to_string(), fields[3]);
    }

    #[test]
    fn input_table_fields_should_return_table_fields_when_none() {
        let input = Input {
            name: None,
            description: None,
            input_type: None,
            default_value: None,
        };
        let fields = input.table_fields();
        assert_eq!(4, fields.len());
        assert_eq!("".to_string(), fields[0]);
        assert_eq!("".to_string(), fields[1]);
        assert_eq!("".to_string(), fields[2]);
        assert_eq!("".to_string(), fields[3]);
    }

    #[test]
    fn enum_table_fields_should_return_table_fields_when_some() {
        let enm = Enum {
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            is_deprecated: Some(true),
            deprecation_reason: Some("meh".to_string()),
        };
        let fields = enm.table_fields();
        assert_eq!(3, fields.len());
        assert_eq!("`name`".to_string(), fields[0]);
        assert_eq!("description".to_string(), fields[1]);
        assert_eq!("meh".to_string(), fields[2]);
    }

    #[test]
    fn enum_table_fields_should_return_table_fields_when_some_and_is_deprecated_is_false() {
        let enm = Enum {
            name: Some("name".to_string()),
            description: Some("description".to_string()),
            is_deprecated: Some(false),
            deprecation_reason: Some("meh".to_string()),
        };
        let fields = enm.table_fields();
        assert_eq!(3, fields.len());
        assert_eq!("`name`".to_string(), fields[0]);
        assert_eq!("description".to_string(), fields[1]);
        assert_eq!("no".to_string(), fields[2]);
    }

    #[test]
    fn enum_table_fields_should_return_table_fields_when_none() {
        let enm = Enum {
            name: None,
            description: None,
            is_deprecated: None,
            deprecation_reason: None,
        };
        let fields = enm.table_fields();
        assert_eq!(3, fields.len());
        assert_eq!("".to_string(), fields[0]);
        assert_eq!("".to_string(), fields[1]);
        assert_eq!("no".to_string(), fields[2]);
    }
}
