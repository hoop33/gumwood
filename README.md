# Gumwood

![badge](https://github.com/hoop33/gumwood/workflows/Rust/badge.svg)

> Convert a GraphQL schema to markdown

## Introduction

Gumwood is alpha and is changing frequently. Not everything documented here works yet.

Its purpose is to prepare a GraphQL schema for publication on a Gatsby or Docusaurus site, or any other site that generates HTML documentation from markdown files.

You specify a live GraphQL endpoint and whether you want the resulting markdown to be stored in a single file or multiple files, and Gumwood will generate the markdown with optional front matter.

## Usage

Get help:

```sh
$ gumwood --help
gumwood 0.1.0
Rob Warner <rwarner@grailbox.com>
Convert a GraphQL schema to Markdown

Specify the source of the schema using --json, --url, or --schema.
 If you don't specify a source, gumwood will read from stdin.
 gumwood will write the markdown files to the current directory,
 unless you specify a different directory using --out-dir.

USAGE:
    gumwood [FLAGS] [OPTIONS]

FLAGS:
    -h, --help        
            Prints help information

    -m, --multiple    
            Splits output into multiple files

    -V, --version     
            Prints version information


OPTIONS:
    -f, --front-matter <front-matter>    
            Front matter for output files

    -H, --header <header>...             
            Header to send in URL request

    -j, --json <json>                    
            File containing introspection response

    -o, --out-dir <out-dir>              
            Output directory [default: .]

    -s, --schema <schema>                
            GraphQL schema file

    -u, --url <url>                      
            URL to introspect
```

**Note:** If you do not specify a source (`--url`, `--json`, or `--schema`), Gumwood will read from `stdin`. This is useful for piping or redirecting your JSON introspection query results into Gumwood. If you don't pipe or redirect anything, Gumwood will wait for you to type your content before continuing.

Convert a GraphQL schema to a single markdown file:

```sh
$ gumwood --url https://example.com/graphql --out-dir /path/to/output
```

Convert a GraphQL schema to multiple markdown files, divided by type:

```sh
$ gumwood --url https://example.com/graphql --out-dir /path/to/output --multiple
```

Convert a GraphQL schema to multiple markdown files, divided by type, with front matter:

```sh
$ gumwood --url https://example.com/graphql --out-dir /path/to/output --multiple \
--front-matter "key1:value1;key2:value2"
```

Convert a GraphQL schema to multiple markdown files, divided by type, when the GraphQL endpoint requires authorization and a user agent:

```sh
$ gumwood --url https://example.com/graphql --out-dir /path/to/output --multiple \
--header "Authorization: bearer myreallylongtoken" --header "User-Agent: gumwood"
```

## Road Map

- [x] Schema load and parse from URL
- [x] Custom headers in URL request
- [x] Generation from an introspection result
- [ ] Generation from a schema file
- [ ] Write to single or multiple files (-m flag)
- [ ] Automatic versioning with semver
- [ ] Automatic releases using GitHub Actions
- [ ] Add front matter to generated file(s)
- [ ] Allow variables in front matter
- [ ] Better error messaging &mdash; maybe a debug mode?
- [x] Objects
- [x] Inputs
- [x] Interfaces
- [x] Enums
- [x] Unions
- [x] Scalars
- [ ] Add links (e.g. from types listed in queries to their actual types)
- [ ] More/better information on Queries markdown
- [ ] More/better information on Mutations markdown
- [ ] More/better information on Subscriptions markdown
- [ ] Optional templates for markdown format
- [ ] More automated testing
- [ ] Code coverage and banner as part of CI/CD

## Contributing

Pull requests and constructive criticism welcome!

### Building

```sh
$ git clone https://github.com/hoop33/gumwood.git && cd gumwood
$ cargo build
```

### Getting Test Code Coverage

Gumwood uses [Tarpaulin](https://github.com/xd009642/tarpaulin) for test code coverage. Per the documentation, Tarpaulin supports only Linux on x86_64.

To use:

```sh
$ make deps # required once only
$ make coverage
```

To get an HTML report:

```sh
$ make html_coverage
```

### Architecture

Gumwood generally follows an MVC pattern:

* Model: `schema.rs`
* View: `schema_markdown.rs` (markdown functions that know about `schema`) and `markdown.rs` (generic markdown functions that know nothing about `schema`)
* Controller: `main.rs`

#### Schema

Responsible for running a GraphQL Introspection query against the provided URL and parsing it into Rust structures that represent the GraphQL schema.

#### Schema Markdown

Responsible for converting a GraphQL schema into opinionated markdown. Stores its result in a HashMap of type => markdown, where type is:

* queries
* mutations
* subscriptions
* objects
* inputs
* interfaces
* enums
* unions
* scalars

Note: that list is cribbed from GitHub's GraphQL documentation <https://docs.github.com/en/graphql/reference> and is subject to change as I better understand the problem space.

#### Markdown

Responsible for generating generic markdown &mdash; utility functions that know nothing about the GraphQL schema.

#### Main

Responsible for:

* Parsing the command-line arguments
* Getting the schema from `schema.rs`
* Getting the markdown from `schema_markdown.rs`
* Writing the markdown file(s)

## FAQ

* Why the name "gumwood"?
    * It's an homage to HoTMaiL &mdash; GuMwooD (G = GraphQL, MD = Markdown)

## Credits

Gumwood uses the following open source libraries &mdash; thank you!

* [reqwest](https://crates.io/crates/reqwest)
* [serde](https://crates.io/crates/serde)
* [serde-json](https://crates.io/crates/serde_json)
* [structopt](https://crates.io/crates/structopt)

Apologies if I've inadvertently omitted any library.

## License

Copyright &copy; 2020 Rob Warner
Licensed under the [MIT License](https://hoop33.mit-license.org/)
