pub mod curl_parsers;

use url::Url;

#[derive(Debug, Clone)]
pub enum Curl {
    Method(String),
    URL(Url),
    Header(String),
    Data(String),
    Flag(String),
}

impl Curl {
    pub fn new(identifier: &str, param: &str) -> Option<Self> {
        if param.is_empty() {
            return None;
        }

        match identifier {
            "-X" => Some(Curl::Method(param.into())),
            "-H" => Some(Curl::Header(param.into())),
            "-d" | "--data" => Some(Curl::Data(param.into())),
            _ => todo!("Haven't implement it yet..."),
        }
    }

    pub fn new_as_flag(identifier: &str) -> Option<Self> {
        // TODO: Do more check to ensure it's a flag param for curl
        Some(Curl::Flag(identifier.into()))
    }

    pub fn new_as_url(url: url::Url) -> Self {
        Curl::URL(url)
    }
}
