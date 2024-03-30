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
            url, fetch, output_dir: dir, user_agent, verbose, add_cookies, cookies_dir, screenshot, ..
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
            let info = get_info(&url, &user_agent);
            let content = fetch_content(fetch, &url, dir, &user_agent, add_cookies, cookies_dir, screenshot, &timestamp);
            let (info, _) = join!(info, content);

            // save info to file
            match info {
                Ok(i) => {
                    let info_json = serde_json::to_string_pretty(&i);
                    match info_json {
                        Ok(json) => {
                            let url_slug = slugify!(&url, stop_words = "https,http,www");
                            let file_name = format!("{url_slug}{timestamp}-info.json");
                            let maybe_file = create_file(dir, file_name);
                            match maybe_file {
                                Ok(mut file) => file.write_all(json.as_bytes())
                                    .expect("Unable to write data"),
                                Err(err) => eprintln!("Error creating file: {}", err),
                            }
                            if *verbose {
                                println!("{}", json);
                            }
                        }
                        Err(err) => eprintln!("Error serializing info: {}", err),
                    }
                }
                Err(err) => eprintln!("Error getting info: {}", err),
            }
        }
    }
    Ok(())
}

async fn get_info(url: &String, user_agent: &String) -> Result<Webpage, Error> {
        
    let mut options = WebpageOptions::default();
    options.allow_insecure = true;
    options.follow_location = true;
    options.max_redirections = 3;
    options.timeout = Duration::from_secs(10);
    options.useragent = user_agent.to_owned();
    
    Webpage::from_url(&*url, options)
}

async fn fetch_content(fetch: &bool,
                       url: &str,
                       dir: &Option<String>,
                       user_agent: &String,
                       add_cookies: &bool,
                       cookies_dir: &Option<String>,
                       screenshot: &bool,
                       timestamp: &String) {
    if !fetch {
        return;
    }
    let maybe_fetch_result = playwright_fetch(
        &url,
        user_agent,
        add_cookies,
        screenshot,
        cookies_dir,
    ).await;
    match maybe_fetch_result {
        Ok(fetch_result) => {
            let file_suffix: &str = match fetch_result.content_type {
                None => "txt",
                Some(content_type) => {
                    if content_type.contains("text/html") {
                        "html"
                    } else if content_type.contains("application/json") {
                        "json"
                    } else {
                        "txt"
                    }
                }
            };
            let url_slug = slugify!(&url, stop_words = "https,http,www");
            let file_name = format!("{url_slug}{timestamp}.{file_suffix}");
            let maybe_file = create_file(dir, file_name);
            match maybe_file {
                Ok(mut file) => {
                    let maybe_content = fetch_result.content;
                    match maybe_content {
                        None => eprintln!("No content found to write to file"),
                        Some(content) => file.write_all(content.as_bytes()).expect("Unable to write data")
                    }
                }
                Err(err) => eprintln!("Error creating file: {}", err),
            }
        }
        Err(err) => eprintln!("Error fetching content: {}", err),
    }
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
        /// Add local cookies to browser context
        #[arg(short, long, default_value = "false")]
        add_cookies: bool,

        /// Cookies directory
        #[arg(short, long, default_value = "./cookies")]
        cookies_dir: Option<String>,

        /// User-agent
        #[arg(short, long, default_value = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36")]
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

        /// Take a screenshot
        #[arg(short, long, default_value = "false")]
        screenshot: bool,

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

/*
await page.evaluate(async () => {
  const delay = ms => new Promise(resolve => setTimeout(resolve, ms));
  for (let i = 0; i < document.body.scrollHeight; i += 100) {
    window.scrollTo(0, i);
    await delay(100);
  }
});
 */

pub struct PlaywrightFetchResult {
    pub content: Option<String>,
    pub content_type: Option<String>,
}

impl PlaywrightFetchResult {
    pub fn new(content: Option<String>, content_type: Option<String>) -> Self {
        Self {
            content,
            content_type,
        }
    }
}

/// Using playwright, fetch the content of the URL
async fn playwright_fetch(url: &str,
                          user_agent: &String,
                          add_cookies: &bool,
                          screenshot: &bool,
                          cookies_dir: &Option<String>) -> Result<PlaywrightFetchResult, Error> {
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
                        cookies.iter().for_each(|cookie| println!("cookie: name={}, value={}", cookie.name, cookie.value));
                        // filter cookies that dont have a value
                        let cookies_to_add: Vec<Cookie> = cookies.into_iter().filter(|cookie| !cookie.value.is_empty()).collect();
                        context.add_cookies(&cookies_to_add).await.expect("Unable to add cookies")
                    }
                }
            }
        }
    }

    context
        .set_extra_http_headers(vec![("Accept-Language".to_string(), "en-US,en;q=0.9".to_string())])
        .await
        .expect("Unable to set extra http headers");

    let page = context.new_page().await.expect("Unable to create page");
    let result = page.goto_builder(url).goto().await;

    let maybe_content_type: Option<String> = match result {
        Ok(maybe_reponse) => {
            match maybe_reponse {
                None => None,
                Some(response) => {
                    match response.headers().await {
                        Ok(headers) => {
                            let maybe_content_type_value = headers.iter()
                                .find(|header| header.name.to_lowercase() == "content-type")
                                .map(|header| header.value.to_string());

                            maybe_content_type_value
                        }
                        Err(_) => None
                    }
                }
            }
        }
        Err(err) => {
            eprintln!("Error fetching page: {}", err);
            None
        }
    };

    let maybe_content = match page.content().await {
        Ok(content) => Some(content),
        Err(_) => {
            eprintln!("Error getting content");
            None
        }
    };


    if *screenshot {
        // TODO: default to using timestamp prefix
        let ss_path = PathBuf::from("screenshot.png");
        page.screenshot_builder()
            .path(ss_path)
            .clear_type()
            .full_page(true)
            .screenshot()
            .await.expect("Unable to take screenshot");
    }

    context.close().await.expect("Unable to close context");
    drop(context);
    Ok(PlaywrightFetchResult::new(maybe_content, maybe_content_type))
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
    pub fn to_playwright_cookie(&self) -> Option<Cookie> {
        if self.key_name.is_empty() {
            return None;
        }
        Some(Cookie {
            name: self.key_name.clone(),
            value: self.value.clone(),
            domain: Option::from(self.host.clone()),
            path: Option::from(self.path.clone()),
            url: None,
            expires: None,
            http_only: Some(self.is_httponly),
            secure: Some(self.is_secure),
            same_site: None,
        })
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
    let playwright_cookies: Vec<Cookie> = hack_cookies.into_iter().filter_map(|cookie| cookie.to_playwright_cookie()).collect();

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
