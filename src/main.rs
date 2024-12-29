use clap::{Command, Parser, Subcommand};
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

#[derive(Parser)]
#[command(name = "nomcurl")]
#[command(version = "0.1.0")]
#[command(about = "A CLI tool to parse and manipulate curl commands")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    #[command(about = "Parses a curl command")]
    Parse {
        /// The input curl command string
        command: String,

        /// Specifies which part of the curl command to parse (method, header, data, flag, url)
        #[arg(short = 'p', long = "part", value_name = "PART")]
        part: Option<CurlCommand>,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { command, part } => match curl_cmd_parse(&command) {
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
        },
        _ => {
            Command::new("nomcurl").print_help().unwrap();
            println!();
        }
    }
}
