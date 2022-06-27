extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::io::Error;
use std::time::Duration;
use clap::{Parser, Subcommand};
use webpage::{Webpage, WebpageOptions};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli: Cli = Cli::parse();
    match &cli.command {
        Command::Info { url } => {
            let options = WebpageOptions {
                allow_insecure: true,
                follow_location: true,
                max_redirections: 2,
                timeout: Duration::from_secs(5),
                ..Default::default()

            };
            let info: Webpage = Webpage::from_url(url, options).expect("Halp, could not fetch");
            let info_json = serde_json::to_string_pretty(&info);
            match info_json {
                Ok(json) => println!("{}", json),
                Err(err) => println!("{}", err),
            }
        }
    }
    Ok(())
}

#[derive(Subcommand)]
enum Command {
    /// Prints information about a webpage
    #[clap(alias = "i")]
    Info {
        /// The URL to interrogate
        url: String
    },
}

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}
