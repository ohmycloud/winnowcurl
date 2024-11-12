use clap::builder::TypedValueParser;
use winnow::ascii::{alpha1, till_line_ending};
use winnow::combinator::{opt, preceded, seq};
use winnow::combinator::{separated, separated_pair};
use winnow::error::InputError;
use winnow::token::take_until;
use winnow::{PResult, Parser};
use super::protocol::Schema;

#[derive(Debug, Clone)]
pub struct QueryString<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct Authority<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug)]
pub struct CurlURL<'a> {
    schema: Schema,
    authority: Option<Authority<'a>>,
    path: &'a str,
    uri: &'a str,
    queries: Option<Vec<QueryString<'a>>>,
    fragment: Option<&'a str>,
}

fn parse_schema<'a>(s: &mut &'a str) -> PResult<Schema, InputError<&'a str>> {
    let mut parser = take_until(1.., ':');
    let schema = parser.parse_next(s)?;
    let schema: Schema = schema.into();
    Ok(schema)
}

fn parse_authority<'a>(s: &mut &'a str) -> PResult<Authority<'a>, InputError<&'a str>> {
    let mut parser = separated_pair(alpha1, ':', alpha1)
        .map(|(username, password)| Authority { username, password });
    preceded("://", parser).parse_next(s)
}

fn parse_auth_part<'a>(s: &mut &'a str) -> PResult<Option<Authority<'a>>, InputError<&'a str>> {
    opt((parse_authority, "@").map(|(auth, _)| auth)).parse_next(s)
}

fn parse_domain<'a>(s: &mut &'a str) -> PResult<&'a str, InputError<&'a str>> {
    let mut parser = take_until(1.., '/');
    if s.starts_with("://") {
        preceded("://", parser).parse_next(s)
    } else {
        parser.parse_next(s)
    }
}

fn parse_uri<'a>(s: &mut &'a str) -> PResult<&'a str, InputError<&'a str>> {
    if s.contains("?") {
        let mut parser = take_until(1.., '?');
        preceded('/', parser).parse_next(s)
    } else {
        preceded('/', till_line_ending).parse_next(s)
    }
}

fn parse_params<'a>(s: &mut &'a str) -> PResult<QueryString<'a>, InputError<&'a str>> {
    separated_pair(take_until(1.., "="), '=', take_until(1.., "&"))
        .map(|(key, value)| QueryString { key, value })
        .parse_next(s)
}

fn parse_query_string<'a>(s: &mut &'a str) -> PResult<Vec<QueryString<'a>>, InputError<&'a str>> {
    separated(1.., parse_params, "&").parse_next(s)
}

fn parse_query_part<'a>(s: &mut &'a str) -> PResult<Option<Vec<QueryString<'a>>>, InputError<&'a str>> {
    opt(("?", parse_query_string).map(|(_, queries)| queries)).parse_next(s)
}

fn parse_fragment<'a>(s: &mut &'a str) -> PResult<Option<&'a str>, InputError<&'a str>> {
    opt(('#', till_line_ending).map(|(_, fragment)| fragment)).parse_next(s)
}

pub fn parse_url<'a>(s: &mut &'a str) -> PResult<CurlURL<'a>, InputError<&'a str>> {
    seq!(
        CurlURL {
            schema: parse_schema,
            authority: parse_auth_part,
            path: parse_domain,
            uri: parse_uri,
            queries: parse_query_part,
            fragment: parse_fragment,
        }
    )
    .parse_next(s)
}
