# Gumwood

> Convert a GraphQL schema to markdown

## Introduction

Gumwood is pre-alpha. It doesn't work yet.

## Usage

Convert a GraphQL file to a single markdown file:

```sh
$ gumwood https://example.com/graphql --out-dir /path/to/output
```

Convert a GraphQL file to multiple markdown files, divided by type:

```sh
$ gumwood https://example.com/graphql --out-dir /path/to/output --multiple
```

Convert a GraphQL file to multiple markdown files, divided by type, with front matter:

```sh
$ gumwood https://example.com/graphql --out-dir /path/to/output --multiple --front-matter "key1:value1;key2:value2"
```

## License

Copyright &copy; 2020 Rob Warner

Licensed under the [MIT License](https://hoop33.mit-license.org/)


