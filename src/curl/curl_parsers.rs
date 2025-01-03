use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::{
        self,
        complete::{alphanumeric0, anychar, char, multispace0, multispace1},
    },
    combinator::{map, map_res, opt, peek, recognize, rest},
    error::{context, Error, ErrorKind},
    multi::fold_many0,
    sequence::{delimited, preceded, tuple},
    IResult,
};

use crate::curl::Curl;

use super::url_parser;

const CURL_CMD: &str = "curl";
pub fn is_curl(input: &str) -> bool {
    input.trim_start().to_lowercase().starts_with(CURL_CMD)
}

pub fn remove_curl_cmd_header(input: &str) -> &str {
    &input[4..]
}

pub fn url_parse(input: &str) -> IResult<&str, Curl> {
    context(
        "url parse",
        preceded(
            multispace0,
            map_res(quoted_data_parse, |d| {
                // let url_parsed = url::Url::parse(d);
                let url_parsed = url_parser::curl_url_parse(d);
                if let Ok((_, u)) = url_parsed {
                    Ok(Curl::new_as_url(u))
                } else {
                    Err(url_parsed)
                }
            }),
        ),
    )(input)
}

/// Identify the ending pattern: <space*>\<space*>\r\n
pub fn slash_line_ending(input: &str) -> IResult<&str, &str> {
    context(
        "Slash line ending",
        recognize(tuple((
            multispace0,
            character::complete::char('\\'),
            multispace0,
        ))),
    )(input)
}

/// Parse double-quoted data with support for escaped characters
fn double_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "Double quoted data parse",
        delimited(
            tuple((multispace0, char('\"'))),
            take_until("\""),
            tuple((char('\"'), multispace0)),
        ),
    )(input)
}

/// Parse single-quoted data with support for escaped characters
fn single_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "Single quoted data parse",
        delimited(
            tuple((multispace0, char('\''))),
            take_until("\'"),
            tuple((char('\''), multispace0)),
        ),
    )(input)
}

/// Get the longest one quoted data between single / double quoted data.
fn quoted_data_parse<'a>(input: &str) -> IResult<&str, &str> {
    let double_res = double_quoted_data_parse(input);
    let single_res = single_quoted_data_parse(input);

    if double_res.is_ok() && single_res.is_ok() {
        let (_double_rest, double_data) = double_res.unwrap();
        let (_single_rest, single_data) = single_res.unwrap();

        if double_data.len() >= single_data.len() {
            Ok((_double_rest, double_data))
        } else {
            Ok((_single_rest, single_data))
        }
    } else if double_res.is_err() && single_res.is_ok() {
        single_res
    } else if double_res.is_ok() && single_res.is_err() {
        double_res
    } else {
        // Both parsing failed, return an error
        let _single_err = single_res.unwrap_err();
        let _double_err = double_res.unwrap_err();

        #[cfg(feature = "debug-print")]
        eprintln!(
            "The origin: ({})\r\nThe single parse error: {}\r\nThe double parse error: {}",
            input, _single_err, _double_err
        );

        Err(nom::Err::Failure(Error::new(&input, ErrorKind::Fail)))
    }
}

pub fn iter_quoted_data_parse(input: &str) -> IResult<&str, Vec<String>> {
    context(
        "Iter quoted data parse",
        fold_many0(
            alt((double_quoted_data_parse, single_quoted_data_parse)),
            Vec::new,
            |mut acc: Vec<String>, item| {
                acc.push(item.into());
                acc
            },
        ),
    )(input)
}

#[macro_export]
macro_rules! parse_command {
    ($name:ident,$($tag:expr),+) => {
        pub fn $name(input: &str) -> IResult<&str, Curl> {
            context(
                stringify!($name),
                preceded(
                    opt(slash_line_ending),
                    map(
                        tuple((multispace0, alt(($( tag($tag) ),+,)), multispace1, quoted_data_parse)),
                        |(_,method, _space, data)| Curl::new(method, data).unwrap(),
                    ),
                ),
            )(input)
        }
    };
}

#[macro_export]
macro_rules! parse_commands {
    ($name:ident,$inner_func:ident) => {
        pub fn $name(input: &str) -> IResult<&str, Vec<Curl>> {
            context(
                stringify!($name),
                fold_many0($inner_func, Vec::new, |mut acc: Vec<Curl>, m| {
                    acc.push(m);
                    acc
                }),
            )(input)
        }
    };
}

parse_command!(method_parse, "-X");
parse_commands!(methods_parse, method_parse);
parse_command!(header_parse, "-H");
parse_commands!(headers_parse, header_parse);
parse_command!(data_parse, "-d", "--data");
parse_commands!(datas_parse, data_parse);
parse_commands!(flags_parse, flag_parse);

pub fn flag_parse(input: &str) -> IResult<&str, Curl> {
    context(
        "flag parse",
        preceded(
            opt(slash_line_ending),
            map_res(
                tuple((
                    tuple((
                        multispace0,
                        character::complete::char('-'),
                        anychar,
                        alphanumeric0,
                    )),
                    peek(rest),
                )),
                |(flag, r)| {
                    let followed_by_data = quoted_data_parse(r);
                    match followed_by_data.is_err() {
                        true => {
                            let (_, a, b, c) = flag;
                            let flag = format!("{}{}{}", a, b, c);
                            match Curl::new_as_flag(&flag) {
                                Some(f) => Ok(f),
                                None => Err(nom::Err::Failure(Error::new(r, ErrorKind::Fail))),
                            }
                        }
                        false => {
                            // Success parsed the quote data
                            Err(nom::Err::Failure(Error::new(r, ErrorKind::Fail)))
                        }
                    }
                },
            ),
        ),
    )(input)
}

pub fn commands_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    context(
        "all commands parse",
        fold_many0(
            alt((method_parse, header_parse, data_parse, flag_parse)),
            Vec::new,
            |mut acc, d| {
                acc.push(d);
                // acc.append(&mut d);
                acc
            },
        ),
    )(input)
}

pub fn curl_cmd_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    if is_curl(input) {
        let mut curl_cmds = Vec::new();
        let input = remove_curl_cmd_header(input.trim_start()); // Remove Curl header firstly
        let url_p = url_parse(input); // Parse the Curl::URL

        let r = match url_p {
            Ok((rest, curl_url)) => {
                curl_cmds.push(curl_url);
                rest
            }
            Err(_) => {
                return Err(nom::Err::Error(Error::new(
                    "No target url found!",
                    ErrorKind::Fail,
                )));
            }
        };

        // Start to extract all command params...
        // For example: -H, -X, -d ...
        let res = context("curl cmd parse", commands_parse)(r);

        if let Ok((_rest, mut cmds)) = res {
            curl_cmds.append(&mut cmds);
            Ok((_rest, curl_cmds))
        } else {
            Err(nom::Err::Failure(Error::new(
                "Fail to parse cmds",
                ErrorKind::Fail,
            )))
        }
    } else {
        Err(nom::Err::Error(Error::new(&input, ErrorKind::Fail)))
    }
}

#[cfg(test)]
mod tests {
    use nom::InputTake;
    // use url::Url;
    use crate::test_util::generic_command_parse;
    use crate::{curl::url_parser, new_curl};

    use super::*;

    trait StrExtensions {
        fn exchange_quotes(&self) -> String;
    }

    impl StrExtensions for str {
        fn exchange_quotes(&self) -> String {
            let mut result = String::with_capacity(self.len());

            for c in self.chars() {
                match c {
                    '"' => result.push('\''),
                    '\'' => result.push('\"'),
                    _ => result.push(c),
                }
            }

            result
        }
    }

    const TEST_CURL_CMD_FULL: &'static str = r#"
        curl 'http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498'  \
        -H 'Accept: */*' -X 'TEST' \
        -H 'Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7' --b \
        -H 'Cache-Control: no-cache' \
        -H 'Connection: keep-alive' \
        -d 'data1:90' \
        --data 'data2:90/i9fi0sdfsdfk\\jfhaoe' \
        -H 'Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}' \
        -H 'Pragma: no-cache' \
        -H 'Referer: http://www.sse.com.cn/'  \
        -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36' \
        --insecure
    "#;

    #[test]
    fn test_curl_cmd_parse() {
        let full_url_str = "http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498";
        // let expect_url = match Url::parse(full_url_str) {
        //     Ok(u) => u,
        //     Err(e) => panic!("Error: {:?}", e),
        // };

        let (_, expect_url) = match url_parser::curl_url_parse(full_url_str) {
            Ok(u) => u,
            Err(e) => panic!("Error: {:?}", e),
        };

        let _url = Curl::new_as_url(expect_url);

        let input = TEST_CURL_CMD_FULL;
        let expect = vec![
            _url,
            new_curl!(-H,"Accept: */*"),
            new_curl!(-X,"TEST"),
            new_curl!(-H,"Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
            new_curl!("--b"),
            new_curl!(-H,"Cache-Control: no-cache"),
            new_curl!(-H,"Connection: keep-alive"),
            new_curl!(-d,"data1:90"),
            new_curl!(-d,"data2:90/i9fi0sdfsdfk\\\\jfhaoe"),
            new_curl!(-H,"Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}"),
            new_curl!(-H,"Pragma: no-cache"),
            new_curl!(-H,"Referer: http://www.sse.com.cn/"),
            new_curl!(-H,"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"),
            new_curl!("--insecure"),
        ];

        generic_command_parse(curl_cmd_parse, input, expect);
    }

    #[test]
    fn test_is_curl() {
        let cmd = "\t \r  \n Curl asdjfnv\n";
        assert!(is_curl(&cmd));
        let cmd = cmd.trim().to_uppercase();
        assert!(is_curl(&cmd));
    }

    #[test]
    fn test_remove_curl_cmd_headr() {
        let cmd = "\t \r  \n Curl asdjfnv\n".trim_start();
        let len = &cmd.len();
        let cmd = remove_curl_cmd_header(cmd);
        assert_eq!(len - 4, cmd.len(), "current cmd is: ({})", cmd);
        assert_ne!("l", cmd.take(1));
        assert_eq!(" ", cmd.take(1));
    }

    #[test]
    fn test_url_parse() {
        let full_url_str = "http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498";
        let (_, expect_url) = match url_parser::curl_url_parse(full_url_str) {
            Ok(u) => u,
            Err(e) => panic!("Error: {:?}", e),
        };

        // let expect_url = match Url::parse(full_url_str) {
        //     Ok(u) => u,
        //     Err(e) => panic!("Error: {:?}", e),
        // };

        let expect = Curl::new_as_url(expect_url);
        let input = format!(" curl \r \t   '{}' \\ \r\n-H 'Accept: */*'", full_url_str);
        let input = remove_curl_cmd_header(&input.trim_start());

        generic_command_parse(url_parse, &input, expect);
    }

    #[test]
    fn test_commands_parse() {
        let expect = vec![
            new_curl!(-H,"Accept: */*"),
            new_curl!(-X,"TEST"),
            new_curl!(-H,"Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7"),
            new_curl!("--b"),
            new_curl!(-H,"Cache-Control: no-cache"),
            new_curl!(-H,"Connection: keep-alive"),
            new_curl!(-d,"data1:90"),
            new_curl!(-d,"data2:90/i9fi0sdfsdfk\\\\jfhaoe"),
            new_curl!(-H,"Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}"),
            new_curl!(-H,"Pragma: no-cache"),
            new_curl!(-H,"Referer: http://www.sse.com.cn/"),
            new_curl!(-H,"User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36"),
            new_curl!("--insecure"),
        ];

        let input = r#"
         \
        -H 'Accept: */*' -X 'TEST' \
        -H 'Accept-Language: en-US,en;q=0.9,zh-CN;q=0.8,zh;q=0.7' --b \
        -H 'Cache-Control: no-cache' \
        -H 'Connection: keep-alive' \
        -d 'data1:90' \
        --data 'data2:90/i9fi0sdfsdfk\\jfhaoe' \
        -H 'Cookie: gdp_user_id=gioenc-c2b256a9%2C5442%2C561b%2C9c02%2C71199e7e89g9; VISITED_MENU=%5B%228312%22%5D; ba17301551dcbaf9_gdp_session_id=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_session_id_sent=2e27fee0-b184-4efa-a66f-f651e5be47e0; ba17301551dcbaf9_gdp_sequence_ids={%22globalKey%22:139%2C%22VISIT%22:4%2C%22PAGE%22:18%2C%22VIEW_CLICK%22:117%2C%22VIEW_CHANGE%22:3}' \
        -H 'Pragma: no-cache' \
        -H 'Referer: http://www.sse.com.cn/'  \
        -H 'User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36' \
        --insecure
        "#;

        generic_command_parse(commands_parse, input, expect);
    }

    #[test]
    fn test_single_quoted_data_parse() {
        let expect = " hhdf,\\fjsdfjl**''";
        let input = format!(
            r##"{}{}"{}" woaini "{}'nmihao'"##,
            "\t \r  \n ", "\n ", expect, " \r \n "
        )
        .exchange_quotes();

        generic_command_parse(single_quoted_data_parse, &input, &expect.exchange_quotes());
    }

    #[test]
    fn test_double_quoted_data_parse() {
        let expect = r#" hhdf,\\fjsdfjl**''"#;
        let input = format!(
            r##"{}{}"{}" woaini "{}'nmihao'"##,
            "\t \r  \n ", "\n ", expect, " \r \n "
        );

        generic_command_parse(double_quoted_data_parse, &input, expect);
    }

    #[test]
    fn test_quoted_data_parse() {
        let expect = " hhdf,\\fjsdfjl**''";
        let input = format!("\t \r  \n \n \"{}\" woaini \" \r \n 'nmihao'", expect);
        generic_command_parse(quoted_data_parse, &input, expect);
    }

    #[test]
    fn test_iter_quoted_data_parse() {
        let expect: Vec<String> = vec![" hhdf,\\fjsdfjl**''".into(), "nmihao".into()];
        let input = format!("\t \r  \n \n \"{}\"   \r \n '{}'", expect[0], expect[1]);

        generic_command_parse(iter_quoted_data_parse, &input, expect);

        let expect = vec![" hhdf,\\fjsdfjl**''".into(), "nmihao".into()];
        let input = format!("\t \r  \n \n \"{}\" \r \n \"{}\"", expect[0], expect[1]);

        generic_command_parse(iter_quoted_data_parse, &input, expect);
    }

    #[test]
    fn test_data_parse() {
        let expect = new_curl!(-d, "AJFjfdslf");
        let input = "\t \r  \n -d \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(data_parse, input, expect);
    }

    #[test]
    fn test_datas_parse() {
        let expect = vec![
            new_curl!(-d, "AJFjfdslf"),
            new_curl!(-d, "abc fjdfl  ii\\hhfjsdkf:90"),
        ];
        let input = "\t \r  \n -d \"AJFjfdslf\" --data \"abc fjdfl  ii\\hhfjsdkf:90\" \t\r jflksfl";
        generic_command_parse(datas_parse, input, expect);
    }

    #[test]
    fn test_method_parse() {
        let expect = new_curl!(-X, "AJFjfdslf");
        let input = "\t \r  \n -X \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(method_parse, input, expect);
    }

    #[test]
    fn test_methods_parse() {
        let expect = vec![
            new_curl!(-X, "AJFjfdslf"),
            new_curl!(-X, "abc fjdfl  ii\\hhfjsdkf:90"),
        ];
        let input = "\t \r  \n -X \"AJFjfdslf\" -X \"abc fjdfl  ii\\hhfjsdkf:90\" \t\r jflksfl";
        generic_command_parse(methods_parse, input, expect);
    }

    #[test]
    fn test_header_parse() {
        let expect = new_curl!(-H, "AJFjfdslf");
        let input = "\t \r  \n -H \"AJFjfdslf\" HHH -H \"llol:90\" -X a";
        generic_command_parse(header_parse, input, expect);
    }

    #[test]
    fn test_headers_parse() {
        let expect = vec![
            new_curl!(-H, "AJFjfdslf"),
            new_curl!(-H, "abc fjdfl  ii\\hhfjsdkf:90"),
        ];
        let input = "\t \r  \n -H \"AJFjfdslf\" -H \"abc fjdfl  ii\\hhfjsdkf:90\" \t\r jflksfl";
        generic_command_parse(headers_parse, input, expect);
    }

    #[test]
    fn test_flag_parse() {
        let expect = new_curl!("--help");
        let input = "\t \r --help -a  \n -X \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(flag_parse, input, expect);
    }

    #[test]
    fn test_flags_parse() {
        let expect = vec![new_curl!("--help"), new_curl!("-a")];
        let input = "\t \r --help -a  \n -X \"AJFjfdslf\" HHH -H \"llol:90\"";
        generic_command_parse(flags_parse, input, expect);
    }
}
