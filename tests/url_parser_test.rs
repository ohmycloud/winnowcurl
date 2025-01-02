use nomcurl::url::parser::parse_url;
use winnow::Located;

#[test]
fn test_parse_with_auth() {
    let input = "https://user:passwd@github.com/rust-lang/rust/issues?labels=E-easy&state=open#ABC";
    let mut input = Located::new(input);
    let url = parse_url(&mut input);

    if let Ok(url) = url {
        println!("{:?}", url);
    }
}

#[test]
fn test_parse_without_auth() {
    let mut input = "https://github.com/rust-lang/rust/issues";
    let mut input = Located::new(input);
    let url = parse_url(&mut input);

    if let Ok(url) = url {
        println!("{:?}", url);
    }
}
