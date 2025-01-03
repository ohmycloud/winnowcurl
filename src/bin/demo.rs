use winnowcurl::curl::curl_parsers::curl_cmd_parse;

fn main() {
    let curl_command = "curl 'http://example.com' -H 'Accept: application/json'";
    let result = curl_cmd_parse(curl_command);
    println!("{:?}", result);
}
