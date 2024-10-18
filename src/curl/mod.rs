pub mod curl_parsers;

use url::Url;

#[macro_export]
macro_rules! new_curl {
    ($identifier:expr) => {
        CurlStru::new($identifier)
    };
    ($identifier:expr,$data:expr) => {
        CurlStru::new_with_data(stringify!($identifier), $data)
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone)]
pub struct CurlStru {
    pub identifier: String,
    pub data: Option<String>,
}

impl CurlStru {
    pub fn new(identifier: &str) -> Self {
        CurlStru {
            identifier: identifier.into(),
            data: None,
        }
    }

    pub fn new_with_data(identifier: &str, data: &str) -> Self {
        CurlStru {
            identifier: identifier.into(),
            data: Some(data.into()),
        }
    }

    pub fn set_data(&mut self, data: Option<String>) {
        self.data = data;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Curl {
    Method(CurlStru),
    URL(Url),
    Header(CurlStru),
    Data(CurlStru),
    Flag(CurlStru),
}

impl Curl {
    pub fn new(identifier: &str, param: &str) -> Option<Self> {
        if param.is_empty() {
            return None;
        }

        match identifier {
            "-X" => Some(Curl::Method(CurlStru::new_with_data(identifier, param))),
            "-H" => Some(Curl::Header(CurlStru::new_with_data(identifier, param))),
            "-d" | "--data" => Some(Curl::Data(CurlStru::new_with_data("-d", param))),
            _ => todo!("Haven't implement it yet..."),
        }
    }

    pub fn new_as_flag(identifier: &str) -> Option<Self> {
        // TODO: Do more check to ensure it's a flag param for curl
        Some(Curl::Flag(CurlStru::new(identifier)))
    }

    pub fn new_as_url(url: url::Url) -> Self {
        Curl::URL(url)
    }
}
