mod markdown;
mod schema;
mod schema_markdown;

use schema::Schema;
use schema_markdown::Markdown;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{self, Read, Write},
    path::PathBuf,
};
use structopt::StructOpt;

/// Convert a GraphQL schema to Markdown
///
/// Specify the source of the schema using --json, --url, or --schema.{n}
/// If you don't specify a source, gumwood will read from stdin.{n}
/// If you specify --out-dir, gumwood will split the output into{n}
/// multiple files by type and write them to the specified directory.{n}
/// If you don't specify --out-dir, gumwood will write to stdout.
#[derive(Debug, StructOpt)]
#[structopt(author)]
pub struct Options {
    #[structopt(short, long, help("URL to introspect"))]
    url: Option<String>,

    #[structopt(
        short,
        long,
        help("File containing introspection response"),
        parse(from_os_str)
    )]
    json: Option<PathBuf>,

    #[structopt(short, long, help("GraphQL schema file"), parse(from_os_str))]
    schema: Option<PathBuf>,

    #[structopt(short = "H", long, help("Header to send in URL request"))]
    header: Vec<String>,

    #[structopt(
        short,
        long,
        help("Output directory for multiple files"),
        parse(from_os_str)
    )]
    out_dir: Option<PathBuf>,

    #[structopt(short, long, help("Front matter for output files"))]
    front_matter: Option<String>,
}

fn get_schema(args: &Options) -> Result<Schema, Box<dyn Error>> {
    let schema: Schema;
    if args.url.is_some() {
        schema = Schema::from_url(&args.url.as_ref().unwrap(), &args.header)?;
    } else if args.json.is_some() {
        schema = Schema::from_json(&args.json.as_ref().unwrap())?;
    } else if args.schema.is_some() {
        schema = Schema::from_schema(&args.schema.as_ref().unwrap())?;
    } else {
        // Read from stdin
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;
        schema = Schema::from_str(&buffer)?;
    }

    Ok(schema)
}

fn write_to_files(
    contents: &HashMap<String, String>,
    out_dir: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    for (name, markdown) in contents {
        if !markdown.is_empty() {
            let out_file = format!("{}.md", name);
            let mut file = File::create(out_dir.join(out_file))?;
            file.write_all(markdown.as_bytes())?;
        }
    }

    Ok(())
}

fn write_to_stdout(contents: &HashMap<String, String>) {
    let mut keys: Vec<_> = contents.keys().collect();
    keys.sort();

    for key in keys.iter() {
        let markdown = contents.get(*key).unwrap();
        if !markdown.is_empty() {
            println!("{}", markdown);
        }
    }
}

/// Takes the arguments from the Options struct and generates
/// markdown for the specified schema.
pub fn run(args: Options) -> Result<(), Box<dyn Error>> {
    let schema = get_schema(&args)?;
    let markdown = Markdown::new(args.out_dir.is_some(), args.front_matter)?;
    let contents = markdown.generate_from_schema(&schema);
    match args.out_dir {
        Some(dir) => write_to_files(&contents, &dir)?,
        None => write_to_stdout(&contents),
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_should_return_ok_when_url_specified() -> Result<(), String> {
        let vec = vec![
            "gumwood",
            "--url",
            "https://example.com",
            "--header",
            "name1:value1",
            "--header",
            "name2:value2",
            "--out-dir",
            "./out",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Options::from_iter(vec.iter());
        assert_eq!(args.url.unwrap(), "https://example.com");
        assert_eq!(args.header.len(), 2);
        assert_eq!(args.header[0], "name1:value1");
        assert_eq!(args.header[1], "name2:value2");
        assert_eq!(
            args.out_dir.unwrap().as_path().display().to_string(),
            "./out"
        );
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        Ok(())
    }

    #[test]
    fn it_should_return_ok_when_json_specified() -> Result<(), String> {
        let vec = vec![
            "gumwood",
            "--json",
            "foo.json",
            "--header",
            "name1:value1",
            "--header",
            "name2:value2",
            "--out-dir",
            "./out",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Options::from_iter(vec.iter());
        assert_eq!(args.json.unwrap().display().to_string(), "foo.json");
        assert_eq!(args.header.len(), 2);
        assert_eq!(args.header[0], "name1:value1");
        assert_eq!(args.header[1], "name2:value2");
        assert_eq!(
            args.out_dir.unwrap().as_path().display().to_string(),
            "./out"
        );
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        Ok(())
    }

    #[test]
    fn it_should_return_ok_when_schema_specified() -> Result<(), String> {
        let vec = vec![
            "gumwood",
            "--schema",
            "schema.graphql",
            "--header",
            "name1:value1",
            "--header",
            "name2:value2",
            "--out-dir",
            "./out",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Options::from_iter(vec.iter());
        assert_eq!(args.schema.unwrap().display().to_string(), "schema.graphql");
        assert_eq!(args.header.len(), 2);
        assert_eq!(args.header[0], "name1:value1");
        assert_eq!(args.header[1], "name2:value2");
        assert_eq!(
            args.out_dir.unwrap().as_path().display().to_string(),
            "./out"
        );
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        Ok(())
    }

    #[test]
    fn it_should_return_error_when_schema_is_passed() {
        let vec = vec!["gumwood", "--schema", "graphql.schema"];
        let args = Options::from_iter(vec.iter());
        assert!(run(args).is_err());
    }

    #[test]
    fn it_should_process_testdata_response_without_error() {
        let vec = vec!["gumwood", "--json", "testdata/response.json"];
        let args = Options::from_iter(vec.iter());
        assert!(run(args).is_ok());
    }
}
