use winnow::ascii::alpha1;
use winnow::combinator::seq;
use winnow::combinator::{separated, separated_pair};
use winnow::error::InputError;
use winnow::token::take_until;
use winnow::{PResult, Parser};

use super::protocol::Protocol;

#[derive(Debug, Clone)]
pub struct QueryString<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct User<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug)]
pub struct CurlURL<'a> {
    protocol: Protocol,
    user: User<'a>,
    domain: &'a str,
    uri: &'a str,
    queries: Vec<QueryString<'a>>,
}

fn parse_protocol<'a>(s: &mut &'a str) -> PResult<Protocol, InputError<&'a str>> {
    let mut parser = take_until(1.., ':');
    let protocol = parser.parse_next(s)?;
    let protocol: Protocol = protocol.into();
    Ok(protocol)
}

fn parse_domain<'a>(s: &mut &'a str) -> PResult<&'a str, InputError<&'a str>> {
    let mut parser = take_until(1.., '/');
    parser.parse_next(s)
}

fn parse_uri<'a>(s: &mut &'a str) -> PResult<&'a str, InputError<&'a str>> {
    take_until(1.., '?').parse_next(s)
}

fn parser_user<'a>(s: &mut &'a str) -> PResult<User<'a>, InputError<&'a str>> {
    separated_pair(alpha1, ':', alpha1)
        .map(|(username, password)| User { username, password })
        .parse_next(s)
}

fn parse_params<'a>(s: &mut &'a str) -> PResult<QueryString<'a>, InputError<&'a str>> {
    separated_pair(take_until(1.., "="), '=', take_until(1.., "&"))
        .map(|(key, value)| QueryString { key, value })
        .parse_next(s)
}

fn parse_query_string<'a>(s: &mut &'a str) -> PResult<Vec<QueryString<'a>>, InputError<&'a str>> {
    separated(1.., parse_params, "&").parse_next(s)
}

fn parse_url<'a>(s: &mut &'a str) -> PResult<CurlURL<'a>, InputError<&'a str>> {
    seq!(
        CurlURL {
            protocol: parse_protocol,
            _: "://",
            user: parser_user,
            _: "@",
            domain: parse_domain,
            _: '/',
            uri: parse_uri,
            _: '?',
            queries: parse_query_string
        }
    )
    .parse_next(s)
}

#[test]
fn url_parser_test() {
    let mut input = "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open";
    let url = parse_url(&mut input);
    if let Ok(url) = url {
        println!("{:?}", url);
    }
}
