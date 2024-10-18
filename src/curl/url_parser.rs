use nom::{
    bytes::complete::{tag, take_till},
    character::{
        self,
        complete::{alpha1, alphanumeric0, alphanumeric1, multispace0},
    },
    combinator::{map, map_res, opt},
    error::{context, Error, ErrorKind},
    sequence::{preceded, tuple},
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
            "smb" => Self::SMB,
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
            // panic!("userinfo should not be empty");
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

    pub fn set_uri(&mut self, uri: &str) -> &mut Self {
        self.uri = Some(uri.into());
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
                opt(uri_parse),
                opt(queries_parse),
                opt(fragment_parse),
            )),
            |(p, d, u, q, f)| {
                let domain = match credentials_domain_to_host_parse(&d) {
                    Ok((_, domain)) => domain,
                    Err(e) => {
                        return Err(e);
                    }
                };

                let mut curl_url = CurlURL::new(&p, domain);

                if let Some(uri) = u {
                    curl_url.set_uri(uri);
                }

                if let Some(queries) = q {
                    let queries = queries_to_query_fragments(queries);
                    curl_url.set_queries(queries);
                }

                if let Some(fragment) = f {
                    curl_url.set_fragment(fragment);
                }

                if let Ok((_, userinfo)) = credentials_domain_to_userinfo_parse(&d) {
                    if let Some(ui) = UserInfo::new(userinfo) {
                        curl_url.set_userinfo(ui);
                    };
                }

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
    // let colon_index = input.find(':');
    let at_index = input.find('@');

    if let Some(at) = at_index {
        // if let Some(colon) = colon_index {
        //     if colon < at {
        //         // It means that :...@ which shows that crediential may in the str
        let userinfo = &input[0..at];
        return IResult::Ok((&input[at + 1..], userinfo));
        //     }
        // }
    }

    IResult::Err(nom::Err::Failure(Error::new(&input, ErrorKind::Fail)))
}

/// Example: github.com
pub fn credentials_domain_to_host_parse(input: &str) -> IResult<&str, &str> {
    let at_index = input.find('@');

    if let Some(at) = at_index {
        // if let Some(colon) = colon_index {
        //     if colon < at {
        //         // It means that :...@ which shows that crediential may in the str
        let userinfo = &input[0..at];
        IResult::Ok((userinfo, &input[at + 1..]))
        //     }
        // }
    } else {
        IResult::Ok(("", input))
    }
}

/// Example: /rust-lang/rust/issues  --> vec![path_fragment]
pub fn uri_parse(input: &str) -> IResult<&str, &str> {
    context("uri_parse", take_till(|c| c == '?'))(input)
}

/// Example: vec![rust-lang,rust,issues]
pub fn uri_to_path_fragments(input: &str) -> Vec<&str> {
    input.split('/').filter(|pf| !pf.is_empty()).collect()
}

/// Example: ?labels=E-easy&state=open --> vec![query_fragment]
pub fn queries_parse(input: &str) -> IResult<&str, &str> {
    context("queries_parse", take_till(|c| c == '#'))(input)
}

/// Example: vec![(labels,E-easy),(state,open)]
pub fn queries_to_query_fragments(input: &str) -> Vec<(String, String)> {
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
    use crate::test_util::{generic_command_parse, generic_parse};

    const TEST_URL_FULL: &'static str =
        "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open#ABC";

    #[test]
    fn test_curl_url_parse() {
        let input = TEST_URL_FULL;
        let userinfo = UserInfo::new("user:passwd").unwrap();
        let queries = queries_to_query_fragments("?labels=E-easy&state=open");
        let mut expect = CurlURL::new("https", "github.com");
        expect
            .set_userinfo(userinfo)
            .set_uri("/rust-lang/rust/issues")
            .set_queries(queries)
            .set_fragment("ABC");

        generic_command_parse(curl_url_parse, &input, expect);

        let input = "http://query.sse.com.cn/commonQuery.do?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498";
        let queries = queries_to_query_fragments("?jsonCallBack=jsonpCallback89469743&sqlId=COMMON_SSE_SJ_GPSJ_CJGK_MRGK_C&PRODUCT_CODE=01%2C02%2C03%2C11%2C17&type=inParams&SEARCH_DATE=2024-03-18&_=1710914422498");
        let mut expect = CurlURL::new("http", "query.sse.com.cn");
        expect.set_uri("/commonQuery.do").set_queries(queries);

        generic_command_parse(curl_url_parse, &input, expect);
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
    fn test_protocol_parse() {
        let input = TEST_URL_FULL;
        let expect = "https";
        generic_command_parse(protocol_parse, input, expect.into());

        let expect = "smb";
        let input = input.replace("https", expect);
        generic_command_parse(protocol_parse, &input, expect.into());

        let expect = "FTP";
        let input = input.replace("smb", expect);
        generic_command_parse(protocol_parse, &input, expect.into());

        let expect = "abc";
        let input = input.replace("FTP", expect);
        generic_command_parse(protocol_parse, &input, expect.into());
    }

    #[test]
    fn test_credentials_domain_parse() {
        let input = TEST_URL_FULL.replace("https://", "");
        let expect = "user:passwd@github.com";
        generic_command_parse(credentials_domain_parse, &input, expect);
    }

    #[test]
    fn test_credentials_domain_to_userinfo_parse() {
        let input = "user:passwd@github.com";
        let expect = "user:passwd";
        generic_command_parse(credentials_domain_to_userinfo_parse, input, expect);
    }

    #[test]
    fn test_credentials_domain_to_host_parse() {
        let input = "user:passwd@github.com";
        let expect = "github.com";
        generic_command_parse(credentials_domain_to_host_parse, input, expect);
    }

    #[test]
    fn test_uri_parse() {
        let input = TEST_URL_FULL.replace("https://user:passwd@github.com", "");
        let expect = "/rust-lang/rust/issues";
        generic_command_parse(uri_parse, &input, expect);
    }

    #[test]
    fn test_uri_to_path_fragments() {
        let input = "/rust-lang/rust/issues";
        let expect = vec!["rust-lang", "rust", "issues"];
        generic_parse(uri_to_path_fragments, input, expect);
    }

    #[test]
    fn test_queries_parse() {
        let input =
            TEST_URL_FULL.replace("https://user:passwd@github.com/rust-lang/rust/issues", "");
        let expect = "?labels=E-easy&state=open";
        generic_command_parse(queries_parse, &input, expect);
    }

    #[test]
    fn test_queries_to_query_fragments() {
        let input = "?labels=E-easy&state=open";
        let expect = vec![
            ("labels".to_string(), "E-easy".to_string()),
            ("state".to_string(), "open".to_string()),
        ];
        generic_parse(queries_to_query_fragments, input, expect);
    }

    #[test]
    fn test_fragment_parse() {
        let input = TEST_URL_FULL.replace(
            "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open",
            "",
        );
        let expect = "ABC";
        generic_command_parse(fragment_parse, &input, expect);
    }
}
