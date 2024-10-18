use clap::{Arg, Command};
use curl::{curl_parsers::curl_cmd_parse, Curl};

pub mod curl;
mod test_util;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CurlCommand {
    Method,
    Header,
    Data,
    Flag,
    Url,
}

// TODO: Build more funcs
fn main() {
    let matches = Command::new("nomcurl")
        .version("0.1.0")
        .about("A CLI tool to parse and manipulate curl commands")
        .subcommand_required(true)
        .arg_required_else_help(true)
        .subcommand(
            Command::new("parse")
                .about("Parses a curl command")
                .arg(
                    Arg::new("command")
                        .help("The input curl command string")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::new("part")
                        .short('p')
                        .long("part")
                        .value_name("PART")
                        .help("Specifies which part of the curl command to parse (method, header, data, flag, url)")
                        .required(false)
                        .value_parser(clap::value_parser!(CurlCommand)),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("parse", sub_matches)) => {
            let command = sub_matches.get_one::<String>("command").unwrap();
            let part = sub_matches.get_one::<CurlCommand>("part");

            match curl_cmd_parse(command) {
                Ok((_remaining, curls)) => {
                    if let Some(part) = part {
                        match part {
                            CurlCommand::Method => {
                                for curl in curls.iter().filter(|c| matches!(c, Curl::Method(_))) {
                                    println!("{:?}", curl);
                                }
                            }
                            CurlCommand::Header => {
                                for curl in curls.iter().filter(|c| matches!(c, Curl::Header(_))) {
                                    println!("{:?}", curl);
                                }
                            }
                            CurlCommand::Data => {
                                for curl in curls.iter().filter(|c| matches!(c, Curl::Data(_))) {
                                    println!("{:?}", curl);
                                }
                            }
                            CurlCommand::Flag => {
                                for curl in curls.iter().filter(|c| matches!(c, Curl::Flag(_))) {
                                    println!("{:?}", curl);
                                }
                            }
                            CurlCommand::Url => {
                                for curl in curls.iter().filter(|c| matches!(c, Curl::URL(_))) {
                                    println!("{:?}", curl);
                                }
                            }
                        }
                    } else {
                        for curl in curls {
                            println!("{:?}", curl);
                        }
                    }
                }
                Err(e) => eprintln!("Error parsing curl command: {:?}", e),
            }
        }
        _ => {
            Command::new("nomcurl").print_help().unwrap();
            println!();
        }
    }
}
