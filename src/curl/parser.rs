use winnow::{
    ascii::multispace0, combinator::delimited, token::take_until, LocatingSlice, ModalResult, Parser,
};

use crate::url::parser::CurlURL;

type Input<'a> = LocatingSlice<&'a str>;

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
        let double_quoted_data = parse_single_quoted_data(&mut input).unwrap();
        assert_eq!(double_quoted_data, expected)
    }
}
