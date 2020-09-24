/// Returns a Markdown header
///
/// # Arguments
///
/// * `level` - The level of the header (i.e. how many leading '#'s)
/// * `text` - The text of the header
pub fn to_header(level: u8, text: &str) -> String {
    format!(
        "{} {}\n\n",
        (0..level).map(|_| "#").collect::<String>(),
        text
    )
}

/// Returns a Markdown description
///
/// # Arguments
///
/// * `text` - the text of the description
pub fn to_description(text: &str) -> String {
    format!("> {}\n\n", text)
}

/// Returns text as Markdown inline code
///
/// # Arguments
///
/// * `text` - the text of the inline code
pub fn to_inline_code(text: &str) -> String {
    if text.is_empty() {
        "".to_string()
    } else {
        format!("`{}`", text)
    }
}

/// Returns a Markdown label and value pair
///
/// # Arguments
///
/// * `label` - the text of the label
/// * `value` - the text of the value
pub fn to_label(label: &str, value: &str) -> String {
    format!("**{}:** {}\n\n", label, value)
}

/// Returns a Markdown link
///
/// # Arguments
///
/// * `text` - the text of the link
/// * `destination` - the destination of the link
pub fn to_link(text: &str, destination: &str) -> String {
    if text.is_empty() {
        "".to_string()
    } else {
        format!("[{}]({})", text, destination)
    }
}

/// Returns a Markdown unordered list
///
/// # Arguments
///
/// * `items` - the text of the items of the list
pub fn to_list(items: &[String]) -> String {
    let list: String = items.iter().map(|item| format!("* {}\n", item)).collect();
    format!("{}\n", list)
}

/// Returns an HTML named anchor (used of intra-document linking)
///
/// # Arguments
///
/// * `text` - the text for the link, which is also used for the anchor
pub fn to_named_anchor(text: &str) -> String {
    format!("<a name=\"{}\"></a>{}", text.to_lowercase(), text)
}

/// Returns a Markdown notice
///
/// # Arguments
///
/// * `notice` - the text of the notice
pub fn to_notice(notice: &str) -> String {
    format!("_{}_\n", notice)
}

/// Returns a markdown table row
///
/// # Arguments
///
/// * `items` - the text of the items (table cells)
pub fn to_table_row(items: &[String]) -> String {
    format!("| {} |\n", items.join(" | "))
}

/// Returns a table separator row
///
/// # Arguments
///
/// * `num` - The number of columns in the table
pub fn to_table_separator(num: usize) -> String {
    to_table_row(&vec!["---".to_string(); num])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn to_header_should_create_header_1() {
        assert_eq!("# My Header\n\n", to_header(1, "My Header"));
    }

    #[test]
    fn to_header_should_create_header_6() {
        assert_eq!("###### My Header\n\n", to_header(6, "My Header"));
    }

    #[test]
    fn to_description_should_create_description() {
        assert_eq!("> My description\n\n", to_description("My description"));
    }

    #[test]
    fn to_inline_code_should_wrap_in_backticks_when_not_empty() {
        assert_eq!("`foo`", to_inline_code("foo"));
    }

    #[test]
    fn to_inline_code_should_return_empty_when_empty() {
        assert_eq!("", to_inline_code(""));
    }

    #[test]
    fn to_label_should_create_label() {
        assert_eq!(
            "**My Label:** My value\n\n",
            to_label("My Label", "My value")
        );
    }

    #[test]
    fn to_link_should_create_link() {
        assert_eq!("[foo](bar)", to_link("foo", "bar"));
    }

    #[test]
    fn to_named_anchor_should_create_named_anchor() {
        assert_eq!("<a name=\"foo\"></a>foo", to_named_anchor("foo"));
    }

    #[test]
    fn to_named_anchor_should_create_named_anchor_when_mixed_case() {
        assert_eq!("<a name=\"foo\"></a>Foo", to_named_anchor("Foo"));
    }

    #[test]
    fn to_notice_should_create_notice() {
        assert_eq!("_My notice_\n", to_notice("My notice"));
    }

    #[test]
    fn to_table_row_should_create_row_when_empty() {
        assert_eq!("|  |\n", to_table_row(&vec![]));
    }

    #[test]
    fn to_table_row_should_create_row_when_not_empty() {
        assert_eq!(
            "| a | b | c |\n",
            to_table_row(&vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }

    #[test]
    fn to_table_separator_should_create_row_when_empty() {
        assert_eq!("|  |\n", to_table_separator(0));
    }

    #[test]
    fn to_table_separator_should_create_row_when_not_empty() {
        assert_eq!("| --- | --- | --- |\n", to_table_separator(3));
    }

    #[test]
    fn to_list_should_return_cr_when_empty() {
        assert_eq!("\n", to_list(&vec![]));
    }

    #[test]
    fn to_list_should_return_list_when_not_empty() {
        assert_eq!(
            "* a\n* b\n* c\n\n",
            to_list(&vec!["a".to_string(), "b".to_string(), "c".to_string()])
        );
    }
}
