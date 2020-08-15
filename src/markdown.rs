use super::schema::Schema;
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

        let query_name = schema.get_query_name();
        println!("{:?}", query_name);

        let mutation_name = schema.get_mutation_name();
        println!("{:?}", mutation_name);

        let subscription_name = schema.get_subscription_name();
        println!("{:?}", subscription_name);

        contents
    }
}

fn header(level: u8, text: &str) -> String {
    // Note that we don't bounds check level -- it's a private function, after all
    format!(
        "{} {}\n\n",
        (0..level).map(|_| "#").collect::<String>(),
        text
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_should_create_header_1() {
        assert_eq!("# My Header\n\n", header(1, "My Header"));
    }

    #[test]
    fn test_header_should_create_header_6() {
        assert_eq!("###### My Header\n\n", header(6, "My Header"));
    }
}
