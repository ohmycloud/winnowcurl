use crate::url::parser::CurlURL;

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
