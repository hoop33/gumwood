mod markdown;
mod schema;
mod schema_markdown;

use schema::Schema;
use schema_markdown::Markdown;
use std::{error::Error, fmt, fs::File, io::Write, path::PathBuf};
use structopt::StructOpt;

#[derive(Debug)]
struct CliError {
    message: String,
}

impl CliError {
    pub fn new(message: &str) -> CliError {
        CliError {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl Error for CliError {}

#[derive(Debug, StructOpt)]
#[structopt(name = "gumwood", about = "Convert a GraphQL schema to Markdown")]
struct Cli {
    #[structopt(short, long, help("The URL to introspect for the GraphQL schema"))]
    url: Option<String>,

    #[structopt(
        short,
        long,
        help("The file containing the JSON response of a GraphQL introspection query"),
        parse(from_os_str)
    )]
    json: Option<PathBuf>,

    #[structopt(short, long, help("The GraphQL schema file"), parse(from_os_str))]
    schema: Option<PathBuf>,

    #[structopt(
        short,
        long,
        help("Header to send in name:value format; allows multiple")
    )]
    header: Vec<String>,

    #[structopt(
        short,
        long,
        help("The output directory for the generated markdown"),
        parse(from_os_str)
    )]
    out_dir: PathBuf,

    #[structopt(short, long, help("Splits output into multiple files"))]
    multiple: bool,

    #[structopt(
        short,
        long,
        help("Front matter to include at the top of output files")
    )]
    front_matter: Option<String>,
}

fn get_schema(args: &Cli) -> Result<Schema, Box<dyn Error>> {
    let schema: Schema;
    if args.url.is_some() {
        schema = Schema::from_url(&args.url.as_ref().unwrap(), &args.header)?;
    } else if args.json.is_some() {
        schema = Schema::from_json(&args.json.as_ref().unwrap())?;
    } else if args.schema.is_some() {
        schema = Schema::from_schema(&args.schema.as_ref().unwrap())?;
    } else {
        return Err(Box::new(CliError::new(
            "you must specify url, json, or schema",
        )));
    }

    Ok(schema)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();

    let schema = get_schema(&args)?;
    let markdown = Markdown::with_front_matter(args.front_matter)?;
    let contents = markdown.generate_from_schema(&schema);
    for (name, markdown) in contents {
        if markdown.len() > 0 {
            let out_file = format!("{}.md", name);
            let mut file = File::create(&args.out_dir.join(out_file))?;
            file.write_all(markdown.as_bytes())?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_it_should_return_ok_when_url_specified() -> Result<(), String> {
        let vec = vec![
            "gumroad",
            "--url",
            "https://example.com",
            "--header",
            "name1:value1",
            "--header",
            "name2:value2",
            "--out-dir",
            "./out",
            "--multiple",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Cli::from_iter(vec.iter());
        assert_eq!(args.url.unwrap(), "https://example.com");
        assert_eq!(args.header.len(), 2);
        assert_eq!(args.header[0], "name1:value1");
        assert_eq!(args.header[1], "name2:value2");
        assert_eq!(args.out_dir.as_path().display().to_string(), "./out");
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        assert!(args.multiple);
        Ok(())
    }

    #[test]
    fn test_it_should_return_ok_when_json_specified() -> Result<(), String> {
        let vec = vec![
            "gumroad",
            "--json",
            "foo.json",
            "--header",
            "name1:value1",
            "--header",
            "name2:value2",
            "--out-dir",
            "./out",
            "--multiple",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Cli::from_iter(vec.iter());
        assert_eq!(args.json.unwrap().display().to_string(), "foo.json");
        assert_eq!(args.header.len(), 2);
        assert_eq!(args.header[0], "name1:value1");
        assert_eq!(args.header[1], "name2:value2");
        assert_eq!(args.out_dir.as_path().display().to_string(), "./out");
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        assert!(args.multiple);
        Ok(())
    }

    #[test]
    fn test_it_should_return_ok_when_schema_specified() -> Result<(), String> {
        let vec = vec![
            "gumroad",
            "--schema",
            "schema.graphql",
            "--header",
            "name1:value1",
            "--header",
            "name2:value2",
            "--out-dir",
            "./out",
            "--multiple",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Cli::from_iter(vec.iter());
        assert_eq!(args.schema.unwrap().display().to_string(), "schema.graphql");
        assert_eq!(args.header.len(), 2);
        assert_eq!(args.header[0], "name1:value1");
        assert_eq!(args.header[1], "name2:value2");
        assert_eq!(args.out_dir.as_path().display().to_string(), "./out");
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        assert!(args.multiple);
        Ok(())
    }

    #[test]
    fn test_multiple_false() -> Result<(), String> {
        let vec = vec![
            "gumroad",
            "--url",
            "https://example.com",
            "--out-dir",
            "./out",
        ];
        let args = Cli::from_iter(vec.iter());
        assert!(!args.multiple);
        Ok(())
    }

    #[test]
    fn test_get_schema_should_return_error_when_none_specified() {
        let vec = vec!["gumroad", "--out-dir", "./out"];
        let args = Cli::from_iter(vec.iter());
        assert!(get_schema(&args).is_err());
    }
}
