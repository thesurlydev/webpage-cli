extern crate clap;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::fs;
use std::fs::File;
use std::io::{Error, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use chrono::Local;
use clap::{Parser, Subcommand};
use playwright::api::Cookie;
use playwright::Playwright;
use serde_derive::{Deserialize, Serialize};
use slugify::slugify;
use tokio::join;
use webpage::{Webpage, WebpageOptions};

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli: Cli = Cli::parse();
    match &cli.command {
        Command::Info {
            url, fetch, output_dir: dir, user_agent, verbose, add_cookies, cookies_dir, ..
        } => {
            if *verbose {
                // print user_agent
                println!("user_agent: {}", user_agent);
                // print url
                println!("url: {}", url);
            }
            let url = handle_scheme(&url);
            if *verbose {
                println!("url(1): {}", url);
            }
            let info = get_info(&url, &user_agent);
            let content = fetch_content(fetch, &url, dir, &user_agent, add_cookies, cookies_dir);
            let (info, _) = join!(info, content);

            match info {
                Ok(i) => {
                    let info_json = serde_json::to_string_pretty(&i);
                    match info_json {
                        Ok(json) => {
                            let url_slug = slugify!(&url, stop_words = "https,http,www");
                            let file_name = format!("{url_slug}-info.json");
                            fs::write(file_name, &json).expect("Unable to write file");
                            if *verbose {
                                println!("{}", json);
                            }
                        },
                        Err(err) => eprintln!("{}", err),
                    }
                }
                Err(err) => eprintln!("Error getting info: {}", err),
            }
        }
    }
    Ok(())
}

async fn get_info(url: &String, user_agent: &String) -> Result<Webpage, Error> {
    let options = WebpageOptions {
        allow_insecure: true,
        follow_location: true,
        max_redirections: 2,
        timeout: Duration::from_secs(5),
        useragent: user_agent.to_owned(),
        ..Default::default()
    };
    Webpage::from_url(&*url, options)
}

async fn fetch_content(fetch: &bool, url: &str, dir: &Option<String>, user_agent: &String, add_cookies: &bool, cookies_dir: &Option<String>) {
    if !fetch {
        return;
    }
    let fetch_result = playwright_fetch(&url, user_agent, add_cookies, cookies_dir).await;
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
        /// Add local cookies to browser context
        #[arg(short, long, default_value = "false")]
        add_cookies: bool,

        /// Cookies directory
        #[arg(short, long, default_value = "./cookies")]
        cookies_dir: Option<String>,

        /// User-agent
        #[arg(short, long, default_value = "Mozilla/5.0 (X11; Linux x86_64; rv:106.0) Gecko/20100101 Firefox/106.0")]
        user_agent: String,

        /// Fetch content via playwright
        #[arg(short, long, default_value = "false")]
        fetch: bool,

        /// Verbose output for debugging
        #[arg(short, long, default_value = "false")]
        verbose: bool,

        /// Directory to save content to
        #[arg(short, long, default_value = "./")]
        output_dir: Option<String>,

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
async fn playwright_fetch(url: &str, user_agent: &String, add_cookies: &bool, cookies_dir: &Option<String>) -> String {
    let playwright = Playwright::initialize().await.expect("Error initializing playwright");
    playwright.prepare().expect("Error installing browsers");
    let chromium = playwright.chromium();
    let browser = chromium.launcher().headless(true).launch().await.expect("Unable to launch browser");

    let context = browser.context_builder()
        .user_agent(user_agent)
        .js_enabled(true)
        .ignore_https_errors(true)
        // .record_har()
        .build().await.expect("Unable to build context");

    if *add_cookies {
        match cookies_dir {
            None => eprintln!("No cookies directory specified"),
            Some(dir) => {
                let results_dir = Some(PathBuf::from(dir));
                let maybe_cookies = get_cookies(url, results_dir).await;
                match maybe_cookies {
                    None => eprintln!("No local cookies found for {}", url),
                    Some(cookies) => {
                        println!("Found {} cookies for url {}; adding to browser context", cookies.len(), url);
                        context.add_cookies(&cookies).await.expect("Unable to add cookies")
                    }
                }
            }
        }
    }

    let page = context.new_page().await.expect("Unable to create page");
    let result = page.goto_builder(url).goto().await;
    /*if result.is_err() {
        eprintln!("Error fetching content: {}", result.err().unwrap());
    }*/
    match result {
        Ok(_) => println!("Successfully fetched content for {}", url),
        Err(err) => {
            eprintln!("Error fetching content: {}", err)
        }
    }
    let maybe_content = page.content().await;
    let content = match maybe_content {
        Ok(content) => content,
        Err(err) => {
            eprintln!("Error getting content: {}", err);
            "".to_string()
        }
    };
    context.close().await.expect("Unable to close context");
    drop(context);
    content
}

/// Represents a cookie from the output of https://github.com/moonD4rk/HackBrowserData
#[derive(Deserialize, Serialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct HackCookie {
    pub host: String,
    pub path: String,
    pub key_name: String,
    pub value: String,
    pub is_secure: bool,
    #[serde(alias = "IsHTTPOnly")]
    pub is_httponly: bool,
    pub has_expire: bool,
    pub is_persistent: bool,
    pub create_date: String,
    pub expire_date: String,
}

impl HackCookie {
    pub fn to_playwright_cookie(&self) -> Cookie {
        Cookie {
            name: self.key_name.clone(),
            value: self.value.clone(),
            domain: Option::from(self.host.clone()),
            path: Option::from(self.path.clone()),
            url: None,
            expires: None,
            http_only: Some(self.is_httponly),
            secure: Some(self.is_secure),
            same_site: None,
        }
    }
}

async fn get_cookies(url: &str, results_dir: Option<PathBuf>) -> Option<Vec<Cookie>> {

    // get host from url
    let url_obj = url::Url::parse(url).expect("error parsing url");
    let host = url_obj.host_str().expect("error getting host from url");
    println!("getting cookies for host: {}", host);

    // determine if results_dir exists
    let results_dir = match results_dir {
        Some(dir) => {
            if dir.exists() {
                dir
            } else {
                PathBuf::from("/tmp")
            }
        }
        None => PathBuf::from("/tmp"),
    };

    let cookie_file = results_dir.join("chrome_default_cookie.json");


    // call out to hack-browser-data-linux-amd64 to decrypt local browsers' cookies
    /*let output = std::process::Command::new("/media/shane/disk1/bin/hack-browser-data-linux-amd64")
        .arg("")
        .output()
        .expect("failed to execute process");*/

    // parse the cookie output json file
    // let cookie_json = String::from_utf8_lossy(&output.stdout);

    // parse the cookies from cookie_file
    let cookie_json = std::fs::read_to_string(cookie_file).expect("error reading cookie file");
    let cookies: Vec<HackCookie> = serde_json::from_str(&*cookie_json).expect("error deserializing cookie json");

    // filter cookies by host
    let hack_cookies: Vec<HackCookie> = cookies.into_iter().filter(|cookie| cookie.host.contains(host)).collect();

    // convert each hackcookie to playwright cookie
    let playwright_cookies: Vec<Cookie> = hack_cookies.into_iter().map(|cookie| cookie.to_playwright_cookie()).collect();

    if playwright_cookies.len() == 0 {
        return None;
    }

    Some(playwright_cookies)
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
