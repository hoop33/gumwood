use std::error::Error;

#[derive(Debug)]
pub struct Markdown {
    front_matter: Option<String>,
}

impl Markdown {
    pub fn with_front_matter(front_matter: Option<String>) -> Result<Markdown, Box<dyn Error>> {
        Ok(Markdown { front_matter })
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
