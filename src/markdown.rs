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

    s
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
                s.push_str(&to_table_row(vec!["Name", "Type", "Kind", "Description"]));
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
                    s.push_str(&to_table_row(vec![&name, &type_name, &kind, &description]));
                }
                s.push_str("\n");
            }
        }
        None => {}
    }

    s
}

// Generic Markdown Conversions

fn to_header(level: u8, text: &str) -> String {
    // Note that we don't bounds check level -- it's a private function, after all
    format!(
        "{} {}\n\n",
        (0..level).map(|_| "#").collect::<String>(),
        text
    )
}

fn to_description(text: &str) -> String {
    format!("> {}\n\n", text)
}

fn to_label(label: &str, value: &str) -> String {
    format!("**{}:** {}\n\n", label, value)
}

fn to_notice(notice: &str) -> String {
    format!("_{}_\n", notice)
}

fn to_table_row(items: Vec<&str>) -> String {
    format!("| {} |\n", items.join(" | "))
}

fn to_table_separator(num: usize) -> String {
    to_table_row(vec!["---"; num])
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
        assert_eq!(3, map.len());
        assert_eq!("".to_string(), map["queries"]);
        assert_eq!("".to_string(), map["mutations"]);
        assert_eq!("".to_string(), map["subscriptions"]);
    }

    // Generic Markdown tests

    #[test]
    fn test_to_header_should_create_header_1() {
        assert_eq!("# My Header\n\n", to_header(1, "My Header"));
    }

    #[test]
    fn test_to_header_should_create_header_6() {
        assert_eq!("###### My Header\n\n", to_header(6, "My Header"));
    }

    #[test]
    fn test_to_description_should_create_description() {
        assert_eq!("> My description\n\n", to_description("My description"));
    }

    #[test]
    fn test_to_label_should_create_label() {
        assert_eq!(
            "**My Label:** My value\n\n",
            to_label("My Label", "My value")
        );
    }

    #[test]
    fn test_to_notice_should_create_notice() {
        assert_eq!("_My notice_\n", to_notice("My notice"));
    }

    #[test]
    fn test_to_table_row_should_create_row_when_empty() {
        assert_eq!("|  |\n", to_table_row(vec![]));
    }

    #[test]
    fn test_to_table_row_should_create_row_when_not_empty() {
        assert_eq!("| a | b | c |\n", to_table_row(vec!["a", "b", "c"]));
    }

    #[test]
    fn test_to_table_separator_should_create_row_when_empty() {
        assert_eq!("|  |\n", to_table_separator(0));
    }

    #[test]
    fn test_to_table_separator_should_create_row_when_not_empty() {
        assert_eq!("| --- | --- | --- |\n", to_table_separator(3));
    }
}
