extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use playwright::Playwright;

use clap::{Parser, Subcommand};
use std::io::{Error, Write};
use std::time::Duration;
use webpage::{Webpage, WebpageOptions};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli: Cli = Cli::parse();
    match &cli.command {
        Command::Info { url, content } => {
            let url: String = if url.starts_with("http://") || url.starts_with("https://") {
                url.to_string()
            } else {
                format!("https://{}", url)
            };

            let options = WebpageOptions {
                allow_insecure: true,
                follow_location: true,
                max_redirections: 2,
                timeout: Duration::from_secs(5),
                ..Default::default()
            };

            let info: Webpage = Webpage::from_url(&*url, options).expect("Halp, could not fetch");
            let info_json = serde_json::to_string_pretty(&info);

            if *content {
                let fetch_result = fetch_content(&url)
                    .await
                    .expect("Error fetching content for ${url}");
                // save string to file
                let mut file =
                    std::fs::File::create("content.html").expect("Unable to create file");
                file.write_all(fetch_result.as_bytes())
                    .expect("Unable to write data");
            }

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
        url: String,

        /// Fetch content via playwright
        #[arg(short, long, default_value = "false")]
        content: bool,
    },
}

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

/// Using playwright, fetch the content of the URL
async fn fetch_content(url: &str) -> Result<String, std::sync::Arc<playwright::Error>> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare().expect("Error installing browsers"); // Install browsers
    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await;
    let context = browser?.context_builder().build().await?;
    let page = context.new_page().await?;
    page.goto_builder(url).goto().await?;
    page.content().await
}
