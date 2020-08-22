use super::schema::{Field, Schema, Type};
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

        match schema.get_query_name() {
            Some(query_name) => {
                let query_type = schema.get_type(&query_name).expect("no query found");
                contents.insert("queries".to_string(), type_to_markdown(&query_type));
            }
            None => {}
        }

        match schema.get_mutation_name() {
            Some(mutation_name) => {
                let mutation_type = schema.get_type(&mutation_name).expect("no mutation found");
                contents.insert("mutations".to_string(), type_to_markdown(&mutation_type));
            }
            None => {}
        }

        match schema.get_subscription_name() {
            Some(subscription_name) => {
                let subscription_type = schema
                    .get_type(&subscription_name)
                    .expect("no subscription found");
                contents.insert(
                    "subscriptions".to_string(),
                    type_to_markdown(&subscription_type),
                );
            }
            None => {}
        }

        contents
    }
}

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
                s.push_str("| Name | Description |\n| ---- | ----------- |\n");
                for arg in args {
                    let name = match &arg.name {
                        Some(name) => name.trim(),
                        None => "(unknown)",
                    };
                    let description = match &arg.description {
                        Some(description) => description.trim().replace("\n", ""),
                        None => "".to_string(),
                    };
                    s.push_str("| ");
                    s.push_str(&name);
                    s.push_str(" | ");
                    s.push_str(&description);
                    s.push_str(" |\n");
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
}
