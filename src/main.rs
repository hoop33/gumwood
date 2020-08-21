mod markdown;
mod schema;

use markdown::Markdown;
use schema::Schema;
use std::error::Error;
use std::fs::File;
use std::io::Write;
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

fn main() -> Result<(), Box<dyn Error>> {
    let args = Cli::from_args();
    let schema = Schema::from_url(&args.url, &args.header)?;
    let markdown = Markdown::with_front_matter(args.front_matter)?;
    let contents = markdown.generate_from_schema(&schema);
    for (name, markdown) in contents {
        let out_file = format!("{}.md", name);
        let mut file = File::create(&args.out_dir.join(out_file))?;
        file.write_all(markdown.as_bytes())?;
    }

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
        assert_eq!(args.url, "https://example.com");
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
        let vec = vec!["gumroad", "https://example.com", "--out-dir", "./out"];
        let args = Cli::from_iter(vec.iter());
        assert!(!args.multiple);
        Ok(())
    }
}
