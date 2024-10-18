use nom::{
    bytes::complete::tag,
    character::{
        self,
        complete::{alpha1, multispace0},
        streaming::alphanumeric0,
    },
    combinator::map,
    error::context,
    sequence::{preceded, terminated, tuple},
    IResult,
};

#[derive(Debug, Clone, PartialEq)]
pub enum Protocol {
    HTTP,
    HTTPS,
    FTP,
    SMB,
    TODO,
}

impl Default for Protocol {
    fn default() -> Self {
        Self::HTTPS
    }
}

impl From<&str> for Protocol {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "http" => Self::HTTP,
            "https" => Self::HTTPS,
            "ftp" => Self::FTP,
            _ => Self::TODO,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInfo(String, String);

/// Example url: "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open#ABC"
#[derive(Debug, Default, Clone, PartialEq)]
pub struct CurlURL {
    pub protocol: Protocol,                     // https
    pub userinfo: Option<UserInfo>,             // user:passwd  -- userinfo --|
    pub domain: String,                         // github.com   -- host     --| --> domain
    pub uri: Option<String>,                    // rust-lang/rust/issues  --> vec![path_fragment]
    pub queries: Option<Vec<(String, String)>>, // ?labels=E-easy&state=open --> vec![query_fragment]
    pub fragment: Option<String>,               // #ABC
}

impl CurlURL {
    pub fn new(protocol: &str, domain: &str) -> Self {
        Self {
            protocol: protocol.into(),
            userinfo: None,
            domain: domain.into(),
            uri: None,
            queries: None,
            fragment: None,
        }
    }

    pub fn set_userinfo(&mut self, userinfo: UserInfo) -> &mut Self {
        self.userinfo = Some(userinfo);
        self
    }

    pub fn set_uri(&mut self, uri: String) -> &mut Self {
        self.uri = Some(uri);
        self
    }

    pub fn set_queries(&mut self, queries: Vec<(String, String)>) -> &mut Self {
        self.queries = Some(queries);
        self
    }

    pub fn set_fragment(&mut self, fragment: &str) -> &mut Self {
        self.fragment = Some(fragment.into());
        self
    }
}

/// Parse the protocol: HTTP/HTTPS/FTP/SMB...
pub fn protocol_parse(input: &str) -> IResult<&str, String> {
    context(
        "protocol_parse",
        preceded(
            multispace0,
            map(
                tuple((
                    alpha1,
                    alphanumeric0,
                    character::complete::char(':'),
                    tag(r#"//"#),
                )),
                |(p1, p2, _, _)| {
                    let protocol = format!("{}{}", p1, p2);
                    protocol
                },
            ),
        ),
    )(input)
}

pub fn domain_parse(input: &str) -> IResult<&str, &str> {
    context("domain_parse", multispace0)(input)
}

pub fn domain_to_userinfo_parse(input: &str) -> IResult<&str, &str> {
    todo!();
}

pub fn domain_to_host_parse(input: &str) -> IResult<&str, &str> {
    todo!();
}

pub fn uri_parse(input: &str) -> IResult<&str, &str> {
    todo!();
}

fn uri_to_path_fragments(input: &str) -> Vec<String> {
    todo!()
}

pub fn queries_parse(input: &str) -> IResult<&str, &str> {
    todo!();
}

fn queries_to_query_fragments(input: &str) -> Vec<(String, String)> {
    todo!()
}

pub fn fragment_parse(input: &str) -> IResult<&str, &str> {
    todo!();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn generic_command_parse<F, I, T>(parser: F, input: I, expect: T)
    where
        F: Fn(I) -> IResult<I, T>,
        T: PartialEq + std::fmt::Debug,
        I: std::fmt::Debug,
    {
        let result = parser(input);
        assert!(result.is_ok(), "The result:\r\n{:#?}", result);
        let (rest, res) = result.unwrap();
        assert_eq!(
            expect, res,
            "The expect:\r\n({:?}) should be same with the result:\r\n({:?})",
            expect, res
        );
    }

    #[test]
    fn test_str_into_protocol() {
        let expect = vec![
            Protocol::HTTP,
            Protocol::HTTPS,
            Protocol::FTP,
            Protocol::SMB,
            Protocol::TODO,
        ];
        let input = vec!["hTtP", "HTTPS", "Ftp", "smB", "asbsdf"];

        let mut result: Vec<Protocol> = Vec::new();
        for p in input {
            result.push(p.into());
        }

        assert_eq!(
            expect, result,
            "The expect:\r\n({:?}) should be same with the result:\r\n({:?})",
            expect, result
        )
    }

    #[test]
    fn test_protocol_parse() {}

    #[test]
    fn test_domain_parse() {}

    #[test]
    fn test_domain_to_userinfo_parse() {}

    #[test]
    fn test_domain_to_host_parse() {}

    #[test]
    fn test_uri_parse() {}

    #[test]
    fn test_uri_to_path_fragments() {}

    #[test]
    fn test_queries_parse() {}

    #[test]
    fn test_queries_to_query_fragments() {}

    #[test]
    fn test_fragment_parse() {}
}
