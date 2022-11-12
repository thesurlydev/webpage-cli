extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::time::Duration;

use clap::{Parser, Subcommand};
use playwright::Playwright;
use tokio::join;
use webpage::{Webpage, WebpageOptions};

use chrono::Local;
use slugify::slugify;

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli: Cli = Cli::parse();
    match &cli.command {
        Command::Info {
            url, fetch, dir, ..
        } => {
            let url = handle_scheme(&url);
            let info = get_info(&url);
            let content = fetch_content(fetch, &url, dir);
            join!(info, content);
        }
    }
    Ok(())
}

async fn get_info(url: &String) {
    let options = WebpageOptions {
        allow_insecure: true,
        follow_location: true,
        max_redirections: 2,
        timeout: Duration::from_secs(5),
        ..Default::default()
    };
    let info: Webpage = Webpage::from_url(&*url, options).expect("Unable to interrogate URL");
    let info_json = serde_json::to_string_pretty(&info);
    match info_json {
        Ok(json) => println!("{}", json),
        Err(err) => println!("{}", err),
    }
}

async fn fetch_content(fetch: &bool, url: &str, dir: &Option<String>) {
    if !fetch {
        return;
    }
    let fetch_result = playwrite_fetch(&url)
        .await
        .expect("Error fetching content for ${url}");
    let url_slug = slugify!(&url, stop_words = "https,http,www");
    let timestamp = Local::now().format("_%Y-%m-%d-%H%M%S");
    let file_name = format!("{url_slug}{timestamp}.html");
    let maybe_file = if dir.is_some() {
        let dir_str = dir.as_ref().unwrap();
        let parent_dir = Path::new(dir_str);
        if !parent_dir.exists() {
            std::fs::create_dir_all(parent_dir).expect("Error creating directory");
        }
        let path = parent_dir.join(file_name);
        File::create(path)
    } else {
        File::create(file_name)
    };

    match maybe_file {
        Ok(mut file) => {
            file.write_all(fetch_result.as_bytes())
                .expect("Unable to write data");
        }
        Err(err) => eprintln!("Error creating file: {}", err),
    }
}

/// Ensure url contains scheme
fn handle_scheme(url: &String) -> String {
    return if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    };
}

#[derive(Subcommand)]
enum Command {
    /// Prints information about a webpage
    #[clap(alias = "i")]
    Info {
        /// Fetch content via playwright
        #[arg(short, long, default_value = "false")]
        fetch: bool,

        /// Directory to save content
        #[arg(short, long, default_value = "./")]
        dir: Option<String>,

        /// The URL to interrogate
        url: String,
    },
}

#[derive(Parser)]
#[clap(version, author, about)]
struct Cli {
    #[clap(subcommand)]
    command: Command,
}

/// Using playwright, fetch the content of the URL
async fn playwrite_fetch(url: &str) -> Result<String, std::sync::Arc<playwright::Error>> {
    let playwright = Playwright::initialize().await?;
    playwright.prepare().expect("Error installing browsers"); // Install browsers
    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await;
    let context = browser?.context_builder().build().await?;
    let page = context.new_page().await?;
    page.goto_builder(url).goto().await?;
    page.content().await
}

mod tests {
    use super::*;

    #[test]
    fn handle_scheme_no_scheme() {
        let url = "example.com".to_string();
        let result = handle_scheme(&url);
        assert_eq!(result, "https://example.com");
    }

    #[test]
    fn handle_scheme_http() {
        let url = "http://example.com".to_string();
        let result = handle_scheme(&url);
        assert_eq!(result, url);
    }

    #[test]
    fn handle_scheme_https() {
        let url = "https://example.com".to_string();
        let result = handle_scheme(&url);
        assert_eq!(result, url);
    }
}
