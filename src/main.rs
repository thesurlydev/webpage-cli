extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::Path;
use std::time::Duration;

use chrono::Local;
use clap::{Parser, Subcommand};
use slugify::slugify;
use webpage::{Webpage, WebpageOptions};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli: Cli = Cli::parse();
    match &cli.command {
        Command::Info {
            url, output_dir: dir, user_agent, verbose, ..
        } => {
            if *verbose {
                // print user_agent
                println!("user_agent: {}", user_agent);
            }
            let url = handle_scheme(&url);
            if *verbose {
                println!("url: {}", url);
            }
            let timestamp = Local::now().format("_%Y-%m-%d-%H%M%S").to_string();
            let info = get_info(&url, &user_agent)?;

            let info_json = serde_json::to_string_pretty(&info);
            match info_json {
                Ok(json) => {
                    // Always print to stdout by default
                    if !*verbose {
                        println!("{}", json);
                    }
                    
                    // Write to file only if output_dir is provided
                    if dir.is_some() {
                        let url_slug = slugify!(&url, stop_words = "https,http,www");
                        let file_name = format!("{url_slug}{timestamp}-info.json");
                        let maybe_file = create_file(dir, file_name);
                        match maybe_file {
                            Ok(mut file) => {
                                file.write_all(json.as_bytes())
                                    .expect("Unable to write data");
                                if *verbose {
                                    println!("Output written to file successfully.");
                                }
                            },
                            Err(err) => eprintln!("Error creating file: {}", err),
                        }
                    }
                }
                Err(err) => eprintln!("Error serializing info: {}", err),
            }

        }
    }
    Ok(())
}

fn get_info(url: &String, user_agent: &String) -> Result<Webpage, Error> {
    let mut options = WebpageOptions::default();
    options.allow_insecure = true;
    options.follow_location = true;
    options.max_redirections = 3;
    options.timeout = Duration::from_secs(10);
    options.useragent = user_agent.to_owned();

    Webpage::from_url(&*url, options)
}


fn create_file(dir: &Option<String>, file_name: String) -> Result<File, Error> {
    let maybe_file = if dir.is_some() {
        let dir_str = dir.as_ref().unwrap();
        let parent_dir = Path::new(dir_str);
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("Error creating directory");
        }
        let path = parent_dir.join(file_name);
        File::create(path)
    } else {
        File::create(file_name)
    };
    maybe_file
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

        /// User-agent
        #[arg(short, long, default_value = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36")]
        user_agent: String,

        /// Verbose output for debugging
        #[arg(short, long, default_value = "false")]
        verbose: bool,

        /// Directory to save content to (if not provided, output is printed to stdout only)
        #[arg(short, long)]
        output_dir: Option<String>,

        #[arg(short, long, default_value = "false")]
        network: bool,

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


#[cfg(test)]
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
