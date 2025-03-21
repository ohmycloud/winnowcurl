use super::protocol::Schema;
use winnow::ascii::alphanumeric1;
use winnow::combinator::{opt, separated, separated_pair, seq};
use winnow::token::{take_until, take_while};
use winnow::{LocatingSlice, ModalResult, Parser};

type Input<'a> = LocatingSlice<&'a str>;

#[derive(Debug, Clone, PartialEq)]
pub struct QueryString<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct Authority<'a> {
    pub username: &'a str,
    pub password: &'a str,
}

#[derive(Debug, PartialEq)]
pub struct CurlURL<'a> {
    pub schema: Schema,
    pub authority: Option<Authority<'a>>,
    pub path: &'a str,
    pub uri: &'a str,
    pub queries: Vec<QueryString<'a>>,
    pub fragment: Option<&'a str>,
}

fn parse_schema<'a>(s: &mut Input<'a>) -> ModalResult<Schema> {
    let schema = take_while(1.., |c| c != ':').parse_next(s)?.into();
    Ok(schema)
}

fn parse_user<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    take_until(1.., ':').parse_next(s)
}

fn prse_password<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    take_until(1.., '@').parse_next(s)
}

fn parse_authority<'a>(s: &mut Input<'a>) -> ModalResult<Authority<'a>> {
    separated_pair(parse_user, ':', prse_password)
        .map(|(username, password)| Authority { username, password })
        .parse_next(s)
}

fn parse_auth_part<'a>(s: &mut Input<'a>) -> ModalResult<Option<Authority<'a>>> {
    opt((parse_authority, "@").map(|(auth, _)| auth)).parse_next(s)
}

fn parse_domain<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c| c != '/').parse_next(s)
}

fn parse_uri<'a>(s: &mut Input<'a>) -> ModalResult<&'a str> {
    take_while(1.., |c| c != '?').parse_next(s)
}

fn param_part<'a>(input: &mut Input<'a>) -> ModalResult<&'a str> {
    take_while(
        1..,
        |c| matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '.' | '_' | '~' | '%' | '+'),
    )
    .parse_next(input)
}

fn parse_params<'a>(s: &mut Input<'a>) -> ModalResult<QueryString<'a>> {
    separated_pair(param_part, '=', param_part)
        .map(|(key, value)| QueryString { key, value })
        .parse_next(s)
}

fn parse_query_part<'a>(s: &mut Input<'a>) -> ModalResult<Vec<QueryString<'a>>> {
    separated(0.., parse_params, "&").parse_next(s)
}

fn parse_fragment<'a>(s: &mut Input<'a>) -> ModalResult<Option<&'a str>> {
    opt(alphanumeric1).parse_next(s)
}

pub fn parse_url<'a>(s: &mut Input<'a>) -> ModalResult<CurlURL<'a>> {
    seq!(CurlURL {
        schema: parse_schema,
        _: "://",
        authority: parse_auth_part,
        path: parse_domain,
        _: opt('/'),
        uri: parse_uri,
        _: opt('?'),
        queries: parse_query_part,
        _: opt('#'),
        fragment: parse_fragment,
    })
    .parse_next(s)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rstest::*;

    #[rstest]
    #[case("https:", Schema::HTTPS)]
    #[case("sftp", Schema::SFTP)]
    fn test_parse_schema(#[case] input: String, #[case] expected: Schema) {
        let mut input = LocatingSlice::new(input.as_str());
        let schema = parse_schema(&mut input).unwrap();
        assert_eq!(schema, expected)
    }

    #[rstest]
    #[case("username:password@github", Authority {username: "username", password: "password"})]
    #[case("admin:aBc%40123@github", Authority {username: "admin", password: "aBc%40123"})]
    fn test_parse_authority(#[case] input: String, #[case] expected: Authority) {
        let mut input = LocatingSlice::new(input.as_str());
        let authority = parse_authority(&mut input).unwrap();
        assert_eq!(authority, expected)
    }

    #[rstest]
    #[case("username:password@github", Some(Authority {username: "username", password: "password"}))]
    #[case("@", None)]
    #[case("abc", None)]
    #[case("abc@", None)]
    fn test_parse_auth_part(#[case] input: String, #[case] expected: Option<Authority>) {
        let mut input = LocatingSlice::new(input.as_str());
        let authority = parse_auth_part(&mut input).unwrap();
        assert_eq!(authority, expected)
    }

    #[rstest]
    #[case("query.sse.com.cn", "query.sse.com.cn")]
    #[case("query.sse.com.cn/", "query.sse.com.cn")]
    fn test_parse_domain(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let domain = parse_domain(&mut input).unwrap();
        assert_eq!(domain, expected)
    }

    #[rstest]
    #[case("rust-lang/rust/issues?", "rust-lang/rust/issues")]
    #[case("rust-lang/rust/issues", "rust-lang/rust/issues")]
    fn test_parse_uri(#[case] input: String, #[case] expected: String) {
        let mut input = LocatingSlice::new(input.as_str());
        let uri = parse_uri(&mut input).unwrap();
        assert_eq!(uri, expected)
    }

    #[rstest]
    #[case("labels=E-easy", QueryString { key: "labels", value: "E-easy" })]
    #[case("labels=E-easy&state=open", QueryString { key: "labels", value: "E-easy" })]
    fn test_parse_params(#[case] input: String, #[case] expected: QueryString) {
        let mut input = LocatingSlice::new(input.as_str());
        let params = parse_params(&mut input).unwrap();
        assert_eq!(params, expected)
    }

    #[rstest]
    #[case("labels=E-easy", vec![QueryString { key: "labels", value: "E-easy" }])]
    #[case(
        "labels=E-easy&state=open",
        vec![
            QueryString { key: "labels", value: "E-easy" },
            QueryString { key: "state", value: "open" }
        ]
    )]
    fn test_parse_query_part(#[case] input: String, #[case] expected: Vec<QueryString>) {
        let mut input = LocatingSlice::new(input.as_str());
        let query = parse_query_part(&mut input).unwrap();
        assert_eq!(query, expected)
    }

    #[rstest]
    #[case("ABC", Some("ABC"))]
    fn test_parse_fragment(#[case] input: String, #[case] expected: Option<&str>) {
        let mut input = LocatingSlice::new(input.as_str());
        let fragment = parse_fragment(&mut input).unwrap();
        assert_eq!(fragment, expected)
    }

    #[rstest]
    #[case(
        "https://github.com/rust-lang/rust/issues",
        CurlURL {
            schema: Schema::HTTPS,
            authority: None,
            path: "github.com",
            uri: "rust-lang/rust/issues",
            queries: vec![],
            fragment: None
        }
    )]
    #[case(
        "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open#ABC",
        CurlURL {
            schema: Schema::HTTPS,
            authority: Some(Authority { username: "user", password: "passwd" }),
            path: "github.com",
            uri: "rust-lang/rust/issues",
            queries: vec![
                QueryString { key: "labels", value: "E-easy" },
                QueryString { key: "state", value: "open" }
            ],
            fragment: Some("ABC")
        }
    )]
    fn test_parse_url(#[case] input: String, #[case] expected: CurlURL) {
        let mut input = LocatingSlice::new(input.as_str());
        let url = parse_url(&mut input).unwrap();
        assert_eq!(url, expected)
    }
}
