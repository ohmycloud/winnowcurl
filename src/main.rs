use clap::{Parser, Subcommand};
use curl::parser::{curl_cmd_parse, Curl};

pub mod curl;
mod test_util;
pub mod url;

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum CurlCommand {
    Method,
    Header,
    Data,
    Flag,
    Url,
}

impl CurlCommand {
    fn matches_curl(&self, curl: &Curl) -> bool {
        match (self, curl) {
            (CurlCommand::Method, Curl::Method(_)) => true,
            (CurlCommand::Header, Curl::Header(_)) => true,
            (CurlCommand::Data, Curl::Data(_)) => true,
            (CurlCommand::Flag, Curl::Flag(_)) => true,
            (CurlCommand::Url, Curl::URL(_)) => true,
            _ => false,
        }
    }
}

#[derive(Parser)]
#[command(name = "winnowcurl")]
#[command(version = "0.1.0")]
#[command(about = "A CLI tool to parse and manipulate curl commands using winnow")]
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
            Ok(curls) => {
                let filtered_curls = curls
                    .iter()
                    .filter(|c| part.map_or(true, |part_type| part_type.matches_curl(c)));
                for curl in filtered_curls {
                    println!("{:?}", curl);
                }
            }
            Err(e) => eprintln!("Error parsing curl command: {}", e),
        },
    }
}
