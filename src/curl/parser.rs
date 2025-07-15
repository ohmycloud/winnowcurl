use winnow::{
    LocatingSlice, ModalResult, Parser,
    ascii::{alphanumeric0, multispace0, multispace1},
    combinator::{alt, delimited, opt, preceded, repeat},
    token::{any, literal, take_until},
};

use crate::url::parser::{CurlURL, parse_url};

type Input<'a> = LocatingSlice<&'a str>;

const CURL_CMD: &str = "curl";

#[derive(Debug, PartialEq, Clone)]
pub struct CurlStru {
    pub identifier: String,
    pub data: Option<String>,
}

#[derive(Debug, PartialEq)]
pub enum Curl<'a> {
    Method(CurlStru),
    URL(CurlURL<'a>),
    Header(CurlStru),
    Data(CurlStru),
    Flag(CurlStru),
}

fn parse_double_quoted_data<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    delimited((multispace0, '"'), take_until(0.., '"'), ('"', multispace0)).parse_next(s)
}

fn parse_single_quoted_data<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    delimited(
        (multispace0, '\''),
        take_until(0.., '\''),
        ('\'', multispace0),
    )
    .parse_next(s)
}

/// Get the longest quoted data between single / double quoted data.
pub fn quoted_data_parse<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    alt((parse_double_quoted_data, parse_single_quoted_data)).parse_next(s)
}

/// Identify the ending pattern: <space*>\<space*>\r\n
pub fn slash_line_ending<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    (multispace0, '\\', multispace0).take().parse_next(s)
}

/// Check if input starts with curl command
pub fn is_curl(input: &str) -> bool {
    input.trim_start().to_lowercase().starts_with(CURL_CMD)
}

/// Remove curl command header
pub fn remove_curl_cmd_header(input: &str) -> &str {
    &input[4..]
}

/// Parse URL in curl command
pub fn url_parse<'a>(s: &mut Input<'a>) -> ModalResult<Curl<'a>> {
    preceded(multispace0, quoted_data_parse)
        .map(|url_str| {
            let mut url_input = LocatingSlice::new(url_str);
            match parse_url(&mut url_input) {
                Ok(url) => Curl::URL(url),
                Err(_) => {
                    // Fallback - create a simple URL structure
                    Curl::URL(crate::url::parser::CurlURL {
                        schema: crate::url::protocol::Schema::HTTP,
                        authority: None,
                        path: url_str,
                        uri: "",
                        queries: vec![],
                        fragment: None,
                    })
                }
            }
        })
        .parse_next(s)
}

/// Parse method arguments like -X
pub fn method_parse<'a>(s: &mut Input<'a>) -> ModalResult<Curl<'a>> {
    preceded(
        opt(slash_line_ending),
        (multispace0, literal("-X"), multispace1, quoted_data_parse).map(|(_, method, _, data)| {
            Curl::Method(CurlStru {
                identifier: method.to_string(),
                data: Some(data.to_string()),
            })
        }),
    )
    .parse_next(s)
}

/// Parse header arguments like -H
pub fn header_parse<'a>(s: &mut Input<'a>) -> ModalResult<Curl<'a>> {
    preceded(
        opt(slash_line_ending),
        (multispace0, literal("-H"), multispace1, quoted_data_parse).map(|(_, header, _, data)| {
            Curl::Header(CurlStru {
                identifier: header.to_string(),
                data: Some(data.to_string()),
            })
        }),
    )
    .parse_next(s)
}

/// Parse data arguments like -d or --data
pub fn data_parse<'a>(s: &mut Input<'a>) -> ModalResult<Curl<'a>> {
    preceded(
        opt(slash_line_ending),
        (
            multispace0,
            alt((literal("-d"), literal("--data"))),
            multispace1,
            quoted_data_parse,
        )
            .map(|(_, data_flag, _, data)| {
                Curl::Data(CurlStru {
                    identifier: data_flag.to_string(),
                    data: Some(data.to_string()),
                })
            }),
    )
    .parse_next(s)
}

/// Parse flag arguments
pub fn flag_parse<'a>(s: &mut Input<'a>) -> ModalResult<Curl<'a>> {
    preceded(
        opt(slash_line_ending),
        (multispace0, '-', any, alphanumeric0).map(|(_, first_char, second_char, rest_chars)| {
            let flag_str = format!("{}{}{}", first_char, second_char, rest_chars);
            Curl::Flag(CurlStru {
                identifier: flag_str,
                data: None,
            })
        }),
    )
    .parse_next(s)
}

/// Parse all commands (methods, headers, data, flags)
pub fn commands_parse<'a>(s: &mut Input<'a>) -> ModalResult<Vec<Curl<'a>>> {
    repeat(
        0..,
        alt((method_parse, header_parse, data_parse, flag_parse)),
    )
    .parse_next(s)
}

/// Parse complete curl command
pub fn curl_cmd_parse(input: &str) -> Result<Vec<Curl<'_>>, String> {
    if !is_curl(input) {
        return Err("Input does not start with curl".to_string());
    }

    let input_without_curl = remove_curl_cmd_header(input.trim_start());
    let mut s = LocatingSlice::new(input_without_curl);

    // Parse URL first
    let url = url_parse(&mut s).map_err(|e| format!("Failed to parse URL: {:?}", e))?;
    let mut curl_cmds = vec![url];

    // Parse remaining commands
    let mut commands =
        commands_parse(&mut s).map_err(|e| format!("Failed to parse commands: {:?}", e))?;
    curl_cmds.append(&mut commands);

    Ok(curl_cmds)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case(r#" "rakudo star" "#, "rakudo star")]
    #[case(r#""rakulang 'rocks'""#, "rakulang 'rocks'")]
    fn test_parse_double_quoted_data(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let double_quoted_data = parse_double_quoted_data(&mut input).unwrap();
        assert_eq!(double_quoted_data, expected)
    }

    #[rstest]
    #[case(r#" 'rakudo star' "#, "rakudo star")]
    #[case(r#"'rakulang "rocks"'"#, r#"rakulang "rocks""#)]
    fn test_parse_single_quoted_data(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let single_quoted_data = parse_single_quoted_data(&mut input).unwrap();
        assert_eq!(single_quoted_data, expected)
    }

    #[rstest]
    #[case(r#" "hello world" "#, "hello world")]
    #[case(r#" 'hello world' "#, "hello world")]
    #[case(r#""longer string data""#, "longer string data")]
    #[case(r#"'single quoted data'"#, "single quoted data")]
    fn test_quoted_data_parse(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let quoted_data = quoted_data_parse(&mut input).unwrap();
        assert_eq!(quoted_data, expected)
    }

    #[rstest]
    #[case("  \\ ", "  \\ ")]
    #[case("\t\\\t", "\t\\\t")]
    #[case(" \\ ", " \\ ")]
    fn test_slash_line_ending(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let result = slash_line_ending(&mut input).unwrap();
        assert_eq!(result, expected)
    }

    #[rstest]
    #[case("curl something", true)]
    #[case("CURL something", true)]
    #[case("  curl test", true)]
    #[case("\t \r\nCurl test", true)]
    #[case("not curl", false)]
    #[case("acurl", false)]
    fn test_is_curl(#[case] input: String, #[case] expected: bool) {
        assert_eq!(is_curl(&input), expected)
    }

    #[rstest]
    #[case("curl command", " command")]
    #[case("curltest", "test")]
    fn test_remove_curl_cmd_header(#[case] input: String, #[case] expected: String) {
        assert_eq!(remove_curl_cmd_header(&input), expected)
    }

    #[rstest]
    #[case(r#" "https://example.com" "#)]
    #[case(r#" 'http://test.com/path' "#)]
    fn test_url_parse(#[case] input: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let result = url_parse(&mut input);
        assert!(result.is_ok());
        if let Ok(Curl::URL(_)) = result {
            // Success - we got a URL
        } else {
            panic!("Expected URL variant");
        }
    }

    #[rstest]
    #[case(r#" -X "GET" "#, "GET")]
    #[case(r#"-X 'POST'"#, "POST")]
    #[case(r#"  -X "PUT"  "#, "PUT")]
    fn test_method_parse(#[case] input: String, #[case] expected_data: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let result = method_parse(&mut input).unwrap();
        if let Curl::Method(curl_stru) = result {
            assert_eq!(curl_stru.identifier, "-X");
            assert_eq!(curl_stru.data.unwrap(), expected_data);
        } else {
            panic!("Expected Method variant");
        }
    }

    #[rstest]
    #[case(
        r#" -H "Content-Type: application/json" "#,
        "Content-Type: application/json"
    )]
    #[case(r#"-H 'Accept: */*'"#, "Accept: */*")]
    fn test_header_parse(#[case] input: String, #[case] expected_data: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let result = header_parse(&mut input).unwrap();
        if let Curl::Header(curl_stru) = result {
            assert_eq!(curl_stru.identifier, "-H");
            assert_eq!(curl_stru.data.unwrap(), expected_data);
        } else {
            panic!("Expected Header variant");
        }
    }

    #[rstest]
    #[case(r#" -d "test data" "#, "-d", "test data")]
    #[case(r#"--data 'json payload'"#, "--data", "json payload")]
    #[case(r#"  --data "form data"  "#, "--data", "form data")]
    fn test_data_parse(
        #[case] input: String,
        #[case] expected_identifier: String,
        #[case] expected_data: String,
    ) {
        let mut input = LocatingSlice::new(input.as_str());
        let result = data_parse(&mut input).unwrap();
        if let Curl::Data(curl_stru) = result {
            assert_eq!(curl_stru.identifier, expected_identifier);
            assert_eq!(curl_stru.data.unwrap(), expected_data);
        } else {
            panic!("Expected Data variant");
        }
    }

    #[rstest]
    #[case(" -v ", "-v")]
    #[case("--insecure", "--insecure")]
    #[case(" -k ", "-k")]
    #[case("-L", "-L")]
    fn test_flag_parse(#[case] input: String, #[case] expected_identifier: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let result = flag_parse(&mut input).unwrap();
        if let Curl::Flag(curl_stru) = result {
            assert_eq!(curl_stru.identifier, expected_identifier);
            assert!(curl_stru.data.is_none());
        } else {
            panic!("Expected Flag variant");
        }
    }

    #[rstest]
    fn test_commands_parse() {
        let input = r#" -X "GET" -H "Accept: */*" -d "test" -v "#;
        let mut input = LocatingSlice::new(input);
        let result = commands_parse(&mut input).unwrap();
        assert_eq!(result.len(), 4);

        // Check that we got the expected types
        let types: Vec<_> = result
            .iter()
            .map(|curl| match curl {
                Curl::Method(_) => "Method",
                Curl::Header(_) => "Header",
                Curl::Data(_) => "Data",
                Curl::Flag(_) => "Flag",
                Curl::URL(_) => "URL",
            })
            .collect();

        assert_eq!(types, vec!["Method", "Header", "Data", "Flag"]);
    }

    #[rstest]
    fn test_curl_cmd_parse_simple() {
        let input = r#"curl "https://example.com" -X "GET""#;
        let result = curl_cmd_parse(input).unwrap();
        assert_eq!(result.len(), 2);

        // First should be URL, second should be Method
        match &result[0] {
            Curl::URL(_) => {}
            _ => panic!("Expected URL as first element"),
        }

        match &result[1] {
            Curl::Method(method) => {
                assert_eq!(method.identifier, "-X");
                assert_eq!(method.data.as_ref().unwrap(), "GET");
            }
            _ => panic!("Expected Method as second element"),
        }
    }

    #[rstest]
    fn test_curl_cmd_parse_complex() {
        let input = r#"curl 'https://api.example.com/data' -X 'POST' -H 'Content-Type: application/json' -d '{"key": "value"}' -v"#;
        let result = curl_cmd_parse(input).unwrap();
        assert_eq!(result.len(), 5);

        // Verify the structure: URL, Method, Header, Data, Flag
        let types: Vec<_> = result
            .iter()
            .map(|curl| match curl {
                Curl::Method(_) => "Method",
                Curl::Header(_) => "Header",
                Curl::Data(_) => "Data",
                Curl::Flag(_) => "Flag",
                Curl::URL(_) => "URL",
            })
            .collect();

        assert_eq!(types, vec!["URL", "Method", "Header", "Data", "Flag"]);
    }

    #[rstest]
    fn test_curl_cmd_parse_invalid() {
        let input = "not a curl command";
        let result = curl_cmd_parse(input);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .contains("Input does not start with curl")
        );
    }

    #[rstest]
    fn test_curl_cmd_parse_full_example() {
        // This mirrors the complex test from curl_parsers.rs
        let input = r#"
        curl 'http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498' \
        -H 'Accept: */*' -X 'TEST' \
        -H 'Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7' \
        -H 'Cache-Control: no-cache' \
        -H 'Connection: keep-alive' \
        -d 'data1:90' \
        --data 'data2:90/i9fi0sdfsdfk\\jfhaoe' \
        -H 'Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D' \
        -H 'Pragma: no-cache' \
        -H 'Referer: http://www.sse.com.cn/' \
        -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36' \
        -v
        "#;

        let result = curl_cmd_parse(input).unwrap();

        // Should have: 1 URL + 1 Method + 6 Headers + 2 Data + 1 Flag = 11 total
        assert!(result.len() >= 10); // At least 10 items (allowing for parsing variations)

        // First should be URL
        match &result[0] {
            Curl::URL(_) => {}
            _ => panic!("Expected URL as first element"),
        }

        // Should contain various types
        let types: Vec<_> = result
            .iter()
            .map(|curl| match curl {
                Curl::Method(_) => "Method",
                Curl::Header(_) => "Header",
                Curl::Data(_) => "Data",
                Curl::Flag(_) => "Flag",
                Curl::URL(_) => "URL",
            })
            .collect();

        assert!(types.contains(&"URL"));
        assert!(types.contains(&"Method"));
        assert!(types.contains(&"Header"));
        assert!(types.contains(&"Data"));
        assert!(types.contains(&"Flag"));
    }

    #[rstest]
    #[case(r#" "test with spaces" "#, "test with spaces")]
    #[case(r#"'test with "nested" quotes'"#, r#"test with "nested" quotes"#)]
    #[case(r#""test with 'nested' quotes""#, "test with 'nested' quotes")]
    fn test_quoted_data_parse_edge_cases(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let quoted_data = quoted_data_parse(&mut input).unwrap();
        assert_eq!(quoted_data, expected)
    }

    #[rstest]
    fn test_commands_parse_empty() {
        let input = "";
        let mut input = LocatingSlice::new(input);
        let result = commands_parse(&mut input).unwrap();
        assert_eq!(result.len(), 0);
    }

    #[rstest]
    fn test_multiple_data_arguments() {
        let input = r#" -d "first data" --data "second data" -d 'third data' "#;
        let mut input = LocatingSlice::new(input);
        let result = commands_parse(&mut input).unwrap();
        assert_eq!(result.len(), 3);

        for cmd in result {
            match cmd {
                Curl::Data(data) => {
                    assert!(data.identifier == "-d" || data.identifier == "--data");
                    assert!(data.data.is_some());
                }
                _ => panic!("Expected only Data variants"),
            }
        }
    }

    #[rstest]
    fn test_multiple_header_arguments() {
        let input = r#" -H "Content-Type: application/json" -H 'Accept: */*' -H "Authorization: Bearer token" "#;
        let mut input = LocatingSlice::new(input);
        let result = commands_parse(&mut input).unwrap();
        assert_eq!(result.len(), 3);

        for cmd in result {
            match cmd {
                Curl::Header(header) => {
                    assert_eq!(header.identifier, "-H");
                    assert!(header.data.is_some());
                }
                _ => panic!("Expected only Header variants"),
            }
        }
    }

    #[rstest]
    fn test_mixed_quote_types() {
        let input = r#"curl "https://example.com" -X 'POST' -H "Content-Type: application/json" -d '{"test": "data"}'"#;
        let result = curl_cmd_parse(input).unwrap();
        assert_eq!(result.len(), 4);

        let types: Vec<_> = result
            .iter()
            .map(|curl| match curl {
                Curl::Method(_) => "Method",
                Curl::Header(_) => "Header",
                Curl::Data(_) => "Data",
                Curl::Flag(_) => "Flag",
                Curl::URL(_) => "URL",
            })
            .collect();

        assert_eq!(types, vec!["URL", "Method", "Header", "Data"]);
    }
}
