# nom-curl: A cURL Command Parser in Rust

[中文](./readme-cn.md) | [English](./readme.md) 

`nom-curl` 是一个用 Rust 编写的库，它使用 `nom` 库来解析 cURL 命令。它可以处理各种 cURL 选项，包括方法、头部、数据和标志。

## Features

- 解析 cURL 命令行选项。
- 支持常见的 cURL 方法，如 `-X`, `-H`, `-d` 等。
- 能够处理带引号的数据和 URL。
- 提供灵活的 API 来扩展新的 cURL 选项。

## Installation

将 `nom-curl` 添加到您的 `Cargo.toml` 文件中：

```toml
[dependencies]
nom-curl = "0.1.8"
```

## Usage

```rust
use nom_curl::parsers::curl_cmd_parse;

fn main() {
    let curl_command = "curl 'http://example.com' -X GET -H 'Accept: application/json'";
    let result = curl_cmd_parse(curl_command);
    println!("{:?}", result);
}
```

## Documentation

详细的 API 文档和指南可以在 [这里](https://docs.rs/nom-curl) 找到。

## Examples

### 解析简单的 cURL 命令

```rust
use nom_curl::parsers::curl_cmd_parse;

let curl_command = "curl 'http://example.com' -X GET";
let result = curl_cmd_parse(curl_command);
assert!(result.is_ok());
```

### 解析带数据的 cURL 命令

```rust
use nom_curl::parsers::curl_cmd_parse;

let curl_command = "curl 'http://example.com' -d 'name=John&age=30'";
let result = curl_cmd_parse(curl_command);
assert!(result.is_ok());
```

## Contributing

我们欢迎任何贡献！请在提交 pull request 前查看 [贡献指南](https://github.com/yourusername/nom-curl/blob/master/CONTRIBUTING.md)。

## License

`nom-curl` 是在 MIT 许可证下发布的。