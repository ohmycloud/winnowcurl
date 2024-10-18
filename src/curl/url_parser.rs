use nom::{
    bytes::complete::{tag, take_till},
    character::{
        self,
        complete::{alpha1, alphanumeric0, alphanumeric1, multispace0},
    },
    combinator::{map, map_res, rest},
    error::context,
    sequence::{preceded, terminated, tuple},
    IResult,
};

const LOCALHOST: &'static str = "localhost";

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

impl UserInfo {
    pub fn new(userinfo: &str) -> Option<Self> {
        if userinfo.is_empty() {
            return None;
            panic!("userinfo should not be empty");
        }
        let mut res = userinfo.splitn(2, ':');
        let name = res.next().unwrap_or("").to_string();
        let pwd = res.next().unwrap_or("").to_string();
        Some(Self(name, pwd))
    }

    pub fn new_explicit(name: &str, pwd: &str) -> Self {
        if name.is_empty() {
            panic!("userinfo should not be empty");
        }
        Self(name.into(), pwd.into())
    }
}

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

/// Parse whole url to entity
pub fn curl_url_parse(input: &str) -> IResult<&str, CurlURL> {
    context(
        "curl_url_parse",
        map_res(
            tuple((
                protocol_parse,
                credentials_domain_parse,
                uri_parse,
                queries_parse,
                fragment_parse,
            )),
            |(p, d, u, q, f)| {
                let userinfo = match credentials_domain_to_userinfo_parse(&d) {
                    Ok((_, userinfo)) => userinfo,
                    Err(e) => {
                        return Err(e);
                    }
                };

                let domain = match credentials_domain_to_host_parse(&d) {
                    Ok((_, domain)) => domain,
                    Err(e) => {
                        return Err(e);
                    }
                };

                let queries = queries_to_query_fragments(q);
                let mut curl_url = CurlURL::new(&p, domain);

                curl_url
                    .set_uri(u.into())
                    .set_queries(queries)
                    .set_fragment(f);

                if let Some(ui) = UserInfo::new(userinfo) {
                    curl_url.set_userinfo(ui);
                };

                Ok(curl_url)
            },
        ),
    )(input)
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

/// Example: user:passwd@github.com
pub fn credentials_domain_parse(input: &str) -> IResult<&str, &str> {
    context("credentials_domain_parse", take_till(|c| c == '/'))(input)
}

/// Example: user:passwd
pub fn credentials_domain_to_userinfo_parse(input: &str) -> IResult<&str, &str> {
    context("domain_to_userinfo_parse", take_till(|c| c == '@'))(input)
}

/// Example: github.com
pub fn credentials_domain_to_host_parse(input: &str) -> IResult<&str, &str> {
    context(
        "domain_to_userinfo_parse",
        preceded(take_till(|c| c == '@'), map(rest, |host: &str| &host[1..])),
    )(input)
}

/// Example: /rust-lang/rust/issues  --> vec![path_fragment]
pub fn uri_parse(input: &str) -> IResult<&str, &str> {
    context("uri_parse", take_till(|c| c == '?'))(input)
}

/// Example: vec![rust-lang,rust,issues]
fn uri_to_path_fragments(input: &str) -> Vec<&str> {
    input.split('/').filter(|pf| !pf.is_empty()).collect()
}

/// Example: ?labels=E-easy&state=open --> vec![query_fragment]
pub fn queries_parse(input: &str) -> IResult<&str, &str> {
    context("queries_parse", take_till(|c| c == '#'))(input)
}

/// Example: vec![(labels,E-easy),(state,open)]
fn queries_to_query_fragments(input: &str) -> Vec<(String, String)> {
    // if '?' exists at the start of queries
    let queries = if input.starts_with('?') {
        &input[1..]
    } else {
        input
    };

    queries
        .split('&')
        .filter(|pf| !pf.is_empty())
        .map(|query| {
            let mut parts = query.splitn(2, '='); // use splitn to set the maxmium splitted items
            let key = parts.next().unwrap_or("");
            let value = parts.next().unwrap_or("");
            (key.into(), value.into())
        })
        .collect()
}

/// Example: #ABC
pub fn fragment_parse(input: &str) -> IResult<&str, &str> {
    context(
        "fragment_parse",
        map(
            tuple((character::complete::char('#'), alphanumeric1)),
            |(_sharp, fragment)| fragment,
        ),
    )(input)
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
