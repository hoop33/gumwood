pub fn to_header(level: u8, text: &str) -> String {
    format!(
        "{} {}\n\n",
        (0..level).map(|_| "#").collect::<String>(),
        text
    )
}

pub fn to_description(text: &str) -> String {
    format!("> {}\n\n", text)
}

pub fn to_label(label: &str, value: &str) -> String {
    format!("**{}:** {}\n\n", label, value)
}

pub fn to_notice(notice: &str) -> String {
    format!("_{}_\n", notice)
}

pub fn to_table_row(items: &Vec<&str>) -> String {
    format!("| {} |\n", items.join(" | "))
}

pub fn to_table_separator(num: usize) -> String {
    to_table_row(&vec!["---"; num])
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!("|  |\n", to_table_row(&vec![]));
    }

    #[test]
    fn test_to_table_row_should_create_row_when_not_empty() {
        assert_eq!("| a | b | c |\n", to_table_row(&vec!["a", "b", "c"]));
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
