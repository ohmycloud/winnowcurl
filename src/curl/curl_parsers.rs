use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, line_ending, multispace0, multispace1, space1},
    combinator::map,
    error::context,
    sequence::{delimited, pair, preceded, terminated, tuple},
    IResult,
};

use super::Curl;

const CURL_CMD: &str = "curl";
pub fn is_curl(input: &str) -> bool {
    input.trim_start().to_lowercase().starts_with(CURL_CMD)
}

pub fn remove_curl_cmd_header(input: &str) -> &str {
    &input[4..]
}

/// Identify the ending pattern: <space+>\<space*>\r\n
pub fn slash_line_ending(input: &str) -> IResult<&str, bool> {
    context(
        "Slash line ending",
        map(
            tuple((multispace1, char('\\'), multispace0, line_ending)),
            |(_, _, _, _)| true,
        ),
    )(input)
}

fn single_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context(
        "Single quoted data parse",
        delimited(char('"'), take_until('"'), char('"')),
    )(input)
}

fn double_quoted_data_parse(input: &str) -> IResult<&str, &str> {
    context("Double quoted data parse", f)(input)
}

fn quoted_data_parse(input: &str) -> IResult<&str, &str> {}

pub fn method_parse(input: &str) -> IResult<&str, Curl> {
    context(
        "Method parse",
        preceded(
            multispace0,
            terminated(map((tuple((tag("-X"), multispace1)), || {}), space1)),
        ),
    )(input)
}

pub fn url_parse(input: &str) -> IResult<&str, Curl> {
    todo!()
}

pub fn headers_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    todo!()
}

pub fn flags_parse(input: &str) -> IResult<&str, Vec<Curl>> {
    todo!()
}

#[cfg(test)]
mod tests {
    use nom::InputTake;

    use super::*;

    #[test]
    fn test_is_curl() {
        let cmd = "\t \r  \n Curl asdjfnv\n";
        assert!(is_curl(&cmd));
        let cmd = cmd.trim().to_uppercase();
        assert!(is_curl(&cmd));
    }

    #[test]
    fn test_method_parse() {
        let cmd = "\t \r  \n Curl asdjfnv\n".trim_start();
        let len = &cmd.len();
        remove_curl_cmd_header(cmd);
        assert_eq!(len - 4, cmd.len());
        assert_ne!("l", cmd.take(1));
        assert_eq!(" ", cmd.take(1))
    }

    #[test]
    fn test_url_parse() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_headers_parse() {
        assert_eq!(1, 1);
    }

    #[test]
    fn test_flags_parse() {
        assert_eq!(1, 1);
    }
}
