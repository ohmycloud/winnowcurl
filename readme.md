# winnow-curl: A cURL Command Parser in Rust

[中文](./readme-cn.md) | [English](./readme.md)

`winnow-curl` is a Rust library for parsing cURL commands using the `winnow` parser combinator library. It can handle various cURL options, including methods, headers, data, and flags.

## Features

- Parse cURL command line options.
- Support for common cURL methods such as `-X`, `-H`, `-d`, etc.
- Ability to handle quoted data and URLs.
- Provides a flexible API for extending new cURL options.

## Installation

Add `winnow-curl` to your `Cargo.toml` file:

```toml
[dependencies]
winnow-curl = "0.1.8"
```

## Usage

```rust
use winnow_curl::parsers::curl_cmd_parse;

fn main() {
    let curl_command = "curl 'http://example.com' -X GET -H 'Accept: application/json'";
    let result = curl_cmd_parse(curl_command);
    println!("{:?}", result);
}
```

## Documentation

For detailed API documentation and guides, visit [here](https://docs.rs/winnow-curl).

## Examples

### Parsing a Simple cURL Command

```rust
use winnow_curl::parsers::curl_cmd_parse;

let curl_command = "curl 'http://example.com' -X GET";
let result = curl_cmd_parse(curl_command);
assert!(result.is_ok());
```

### Parsing a cURL Command with Data

```rust
use winnow_curl::parsers::curl_cmd_parse;

let curl_command = "curl 'http://example.com' -d 'name=John&age=30'";
let result = curl_cmd_parse(curl_command);
assert!(result.is_ok());
```

## Contributing

We welcome any contributions! Please review the [contribution guidelines](https://github.com/yourusername/winnow-curl/blob/master/CONTRIBUTING.md) before submitting a pull request.

## License

`winnow-curl` is released under the MIT license.
