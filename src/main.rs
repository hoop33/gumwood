mod markdown;
mod schema;

use markdown::Markdown;
use schema::Schema;
use std::error::Error;
use std::path::PathBuf;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "gumwood", about = "Convert a GraphQL schema to Markdown")]
struct Cli {
    #[structopt()]
    url: String,

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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();
    let schema = Schema::from_url(&args.url)?;
    let markdown = Markdown::with_front_matter(args.front_matter)?;

    match schema.query_type {
        Some(query_type) => println!("query type: {}", query_type),
        None => {}
    }

    println!("mutation type: {:?}", schema.mutation_type);
    println!("{:?}", markdown);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_options() -> Result<(), String> {
        let vec = vec![
            "gumroad",
            "https://example.com",
            "--out-dir",
            "./out",
            "--multiple",
            "--front-matter",
            "a:b;c:d",
        ];
        let args = Cli::from_iter(vec.iter());
        assert_eq!(args.url, "https://example.com");
        assert_eq!(args.out_dir.as_path().display().to_string(), "./out");
        assert_eq!(args.front_matter.unwrap(), "a:b;c:d");
        assert!(args.multiple);
        Ok(())
    }

    #[test]
    fn test_multiple_false() -> Result<(), String> {
        let vec = vec!["gumroad", "https://example.com", "--out-dir", "./out"];
        let args = Cli::from_iter(vec.iter());
        assert!(!args.multiple);
        Ok(())
    }
}
