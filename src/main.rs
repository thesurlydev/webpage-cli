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

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/115.0.0.0 Safari/537.36";

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli: Cli = Cli::parse();
    match &cli.command {
        Command::Info {
            url,
            output_dir: dir,
            user_agent,
            verbose,
            ..
        } => {
            if *verbose {
                // print user_agent
                println!("user_agent: {}", user_agent);
            }
            let url = handle_scheme(url);
            if *verbose {
                println!("url: {}", url);
            }
            let timestamp = Local::now().format("_%Y-%m-%d-%H%M%S").to_string();
            let info = get_info(&url, user_agent)?;

            let info_json = serde_json::to_string_pretty(&info);
            match info_json {
                Ok(json) => {
                    // Always print to stdout by default
                    if !*verbose {
                        println!("{}", json);
                    }

                    // Write to file only if output_dir is provided
                    if dir.is_some() {
                        let file_name = info_file_name(&url, &timestamp);
                        let maybe_file = create_file(dir, file_name);
                        match maybe_file {
                            Ok(mut file) => {
                                file.write_all(json.as_bytes())
                                    .expect("Unable to write data");
                                if *verbose {
                                    println!("Output written to file successfully.");
                                }
                            }
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

fn get_info(url: &str, user_agent: &str) -> Result<Webpage, Error> {
    let mut options = WebpageOptions::default();
    options.allow_insecure = true;
    options.follow_location = true;
    options.max_redirections = 3;
    options.timeout = Duration::from_secs(10);
    options.useragent = user_agent.to_owned();

    Webpage::from_url(url, options)
}

fn info_file_name(url: &str, timestamp: &str) -> String {
    let url_slug = slugify!(url, stop_words = "https,http,www");
    format!("{url_slug}{timestamp}-info.json")
}

fn create_file(dir: &Option<String>, file_name: String) -> Result<File, Error> {
    if let Some(dir_str) = dir {
        let parent_dir = Path::new(dir_str);
        if !parent_dir.exists() {
            fs::create_dir_all(parent_dir).expect("Error creating directory");
        }
        File::create(parent_dir.join(file_name))
    } else {
        File::create(file_name)
    }
}

/// Ensure url contains scheme
fn handle_scheme(url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else {
        format!("https://{}", url)
    }
}

#[derive(Subcommand)]
enum Command {
    /// Prints information about a webpage
    #[clap(alias = "i")]
    Info {
        /// User-agent
        #[arg(short, long, default_value = DEFAULT_USER_AGENT)]
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

    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::path::PathBuf;
    use std::thread;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_suffix() -> String {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        format!("{}-{}", std::process::id(), nanos)
    }

    fn unique_temp_dir(label: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("webpage-cli-test-{label}-{}", unique_suffix()));
        fs::create_dir_all(&dir).expect("could not create temp dir");
        dir
    }

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

    #[test]
    fn handle_scheme_no_scheme_with_path_and_query() {
        let url = "example.com/path?q=1".to_string();
        assert_eq!(handle_scheme(&url), "https://example.com/path?q=1");
    }

    #[test]
    fn info_file_name_strips_scheme_and_www() {
        assert_eq!(
            info_file_name("https://www.example.com/foo", "_2026-10-25-120000"),
            "example-com-foo_2026-10-25-120000-info.json"
        );
    }

    #[test]
    fn info_file_name_handles_bare_domain() {
        assert_eq!(
            info_file_name("https://example.com", "_2026-10-25-120000"),
            "example-com_2026-10-25-120000-info.json"
        );
    }

    #[test]
    fn info_file_name_flattens_query_and_fragment() {
        assert_eq!(
            info_file_name("http://example.com/path?a=1#top", "_2026-10-25-120000"),
            "example-com-path-a-1-top_2026-10-25-120000-info.json"
        );
    }

    #[test]
    fn info_file_name_transliterates_unicode() {
        assert_eq!(
            info_file_name("https://example.com/caf\u{e9}", "_2026-10-25-120000"),
            "example-com-cafe_2026-10-25-120000-info.json"
        );
    }

    #[test]
    fn create_file_uses_existing_directory() {
        let base = unique_temp_dir("existing-dir");
        let dir = Some(base.to_string_lossy().into_owned());

        let mut file = create_file(&dir, "info.json".to_string()).expect("could not create file");
        file.write_all(b"data").expect("could not write file");
        drop(file);

        assert_eq!(fs::read_to_string(base.join("info.json")).unwrap(), "data");
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn create_file_creates_missing_directories() {
        let base = unique_temp_dir("missing-dirs");
        let nested = base.join("a").join("b");
        let dir = Some(nested.to_string_lossy().into_owned());

        let mut file = create_file(&dir, "info.json".to_string()).expect("could not create file");
        file.write_all(b"nested").expect("could not write file");
        drop(file);

        assert_eq!(
            fs::read_to_string(nested.join("info.json")).unwrap(),
            "nested"
        );
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn create_file_overwrites_existing_file() {
        let base = unique_temp_dir("overwrite");
        fs::write(base.join("info.json"), "old and longer contents").unwrap();
        let dir = Some(base.to_string_lossy().into_owned());

        let mut file = create_file(&dir, "info.json".to_string()).expect("could not create file");
        file.write_all(b"new").expect("could not write file");
        drop(file);

        assert_eq!(fs::read_to_string(base.join("info.json")).unwrap(), "new");
        fs::remove_dir_all(&base).unwrap();
    }

    #[test]
    fn create_file_without_directory_uses_current_directory() {
        let file_name = format!("webpage-cli-test-{}.json", unique_suffix());

        let mut file = create_file(&None, file_name.clone()).expect("could not create file");
        file.write_all(b"cwd").expect("could not write file");
        drop(file);

        assert_eq!(fs::read_to_string(&file_name).unwrap(), "cwd");
        fs::remove_file(&file_name).unwrap();
    }

    struct InfoArgs {
        url: String,
        user_agent: String,
        verbose: bool,
        output_dir: Option<String>,
        network: bool,
    }

    fn parse_info(args: &[&str]) -> InfoArgs {
        let cli = Cli::try_parse_from(args).expect("arguments should parse");
        match cli.command {
            Command::Info {
                url,
                user_agent,
                verbose,
                output_dir,
                network,
            } => InfoArgs {
                url,
                user_agent,
                verbose,
                output_dir,
                network,
            },
        }
    }

    #[test]
    fn cli_info_defaults() {
        let parsed = parse_info(&["webpage-cli", "info", "example.com"]);
        assert_eq!(parsed.url, "example.com");
        assert_eq!(parsed.user_agent, DEFAULT_USER_AGENT);
        assert!(!parsed.verbose);
        assert!(!parsed.network);
        assert_eq!(parsed.output_dir, None);
    }

    #[test]
    fn cli_info_alias() {
        let parsed = parse_info(&["webpage-cli", "i", "example.com"]);
        assert_eq!(parsed.url, "example.com");
    }

    #[test]
    fn cli_info_accepts_short_flags() {
        let parsed = parse_info(&[
            "webpage-cli",
            "info",
            "-u",
            "custom-agent",
            "-v",
            "-n",
            "-o",
            "/tmp/out",
            "example.com",
        ]);
        assert_eq!(parsed.user_agent, "custom-agent");
        assert!(parsed.verbose);
        assert!(parsed.network);
        assert_eq!(parsed.output_dir.as_deref(), Some("/tmp/out"));
        assert_eq!(parsed.url, "example.com");
    }

    #[test]
    fn cli_info_accepts_long_flags() {
        let parsed = parse_info(&[
            "webpage-cli",
            "info",
            "--user-agent",
            "custom-agent",
            "--verbose",
            "--network",
            "--output-dir",
            "/tmp/out",
            "example.com",
        ]);
        assert_eq!(parsed.user_agent, "custom-agent");
        assert!(parsed.verbose);
        assert!(parsed.network);
        assert_eq!(parsed.output_dir.as_deref(), Some("/tmp/out"));
    }

    #[test]
    fn cli_info_requires_url() {
        assert!(Cli::try_parse_from(["webpage-cli", "info"]).is_err());
    }

    #[test]
    fn cli_rejects_unknown_subcommand() {
        assert!(Cli::try_parse_from(["webpage-cli", "bogus"]).is_err());
    }

    #[test]
    fn cli_prints_version() {
        let err = match Cli::try_parse_from(["webpage-cli", "--version"]) {
            Ok(_) => panic!("--version should not parse into a command"),
            Err(err) => err,
        };
        assert_eq!(err.kind(), clap::error::ErrorKind::DisplayVersion);
    }

    /// Bind a local HTTP server that serves `html` to exactly one request.
    ///
    /// Returns the URL to fetch and a handle yielding the raw request text so
    /// tests can assert on headers that were actually sent.
    fn serve_once(html: &'static str) -> (String, thread::JoinHandle<String>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("could not bind listener");
        let addr = listener
            .local_addr()
            .expect("could not read listener address");

        let handle = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("could not accept connection");
            let mut reader = BufReader::new(stream.try_clone().expect("could not clone stream"));
            let mut request = String::new();
            loop {
                let mut line = String::new();
                if reader.read_line(&mut line).expect("could not read request") == 0 {
                    break;
                }
                request.push_str(&line);
                if line == "\r\n" {
                    break;
                }
            }

            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                html.len(),
                html
            );
            stream
                .write_all(response.as_bytes())
                .expect("could not write response");
            stream.flush().expect("could not flush response");
            request
        });

        (format!("http://{addr}"), handle)
    }

    #[test]
    fn get_info_fetches_and_parses_page() {
        let html = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>Test Page</title>
  <meta name="description" content="A page for testing">
</head>
<body><p>Hello from the local test server.</p></body>
</html>"#;
        let (url, handle) = serve_once(html);

        let info = get_info(&url, "webpage-cli-test-agent").expect("could not fetch local page");

        assert_eq!(info.http.response_code, 200);
        assert!(info.http.content_type.starts_with("text/html"));
        // curl normalizes a bare origin URL by appending a trailing slash.
        assert_eq!(info.http.url.trim_end_matches('/'), url);
        assert_eq!(info.html.title.as_deref(), Some("Test Page"));
        assert_eq!(info.html.description.as_deref(), Some("A page for testing"));
        assert_eq!(info.html.language.as_deref(), Some("en"));
        assert!(
            info.html
                .text_content
                .contains("Hello from the local test server.")
        );

        let json = serde_json::to_string(&info).expect("info should serialize");
        assert!(json.contains("\"title\":\"Test Page\""));

        let request = handle.join().expect("server thread panicked");
        assert!(
            request
                .to_lowercase()
                .contains("user-agent: webpage-cli-test-agent")
        );
    }

    #[test]
    fn get_info_rejects_malformed_url() {
        let result = get_info("not a url", "webpage-cli-test-agent");
        assert!(result.is_err());
    }

    #[test]
    fn get_info_returns_error_when_connection_is_refused() {
        // Binding and immediately dropping a listener gives us a port that is
        // very likely closed, without depending on any external network.
        let listener = TcpListener::bind("127.0.0.1:0").expect("could not bind listener");
        let addr = listener
            .local_addr()
            .expect("could not read listener address");
        drop(listener);

        let result = get_info(&format!("http://{addr}"), "webpage-cli-test-agent");
        assert!(result.is_err());
    }
}
