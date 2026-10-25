use std::fs;
use std::io::{BufRead, BufReader, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::{Instant, SystemTime, UNIX_EPOCH};

const PAGE: &str = r#"<!DOCTYPE html>
<html lang="en">
<head>
  <title>Test Page</title>
  <meta name="description" content="A page for testing">
</head>
<body><p>Hello from the local test server.</p></body>
</html>"#;

fn unique_suffix() -> String {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    format!("{}-{}", std::process::id(), nanos)
}

fn unique_temp_dir(label: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("webpage-cli-e2e-{label}-{}", unique_suffix()));
    fs::create_dir_all(&dir).expect("could not create temp dir");
    dir
}

/// Bind a local HTTP server that serves `html` to exactly one request.
///
/// The accept loop polls with a deadline so a misbehaving binary fails the
/// test instead of hanging the suite forever. Returns the URL to fetch and a
/// handle yielding the raw request text so tests can assert on sent headers.
fn serve_once(html: &'static str) -> (String, thread::JoinHandle<String>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("could not bind listener");
    listener
        .set_nonblocking(true)
        .expect("could not make listener non-blocking");
    let addr = listener
        .local_addr()
        .expect("could not read listener address");

    let handle = thread::spawn(move || {
        let deadline = Instant::now() + std::time::Duration::from_secs(10);
        let (mut stream, _) = loop {
            match listener.accept() {
                Ok(connection) => break connection,
                Err(err) if err.kind() == std::io::ErrorKind::WouldBlock => {
                    assert!(Instant::now() < deadline, "timed out waiting for request");
                    thread::sleep(std::time::Duration::from_millis(10));
                }
                Err(err) => panic!("could not accept connection: {err}"),
            }
        };
        stream
            .set_nonblocking(false)
            .expect("could not make stream blocking");

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

fn run_cli(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_webpage-cli"))
        .args(args)
        .output()
        .expect("could not run webpage-cli")
}

#[test]
fn info_prints_json_for_a_fetched_page() {
    let (url, handle) = serve_once(PAGE);

    let output = run_cli(&["info", &url]);

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON");
    assert_eq!(json["http"]["response_code"], 200);
    assert_eq!(json["html"]["title"], "Test Page");
    assert_eq!(json["html"]["description"], "A page for testing");

    // The default user agent should be what actually reaches the server.
    let request = handle.join().expect("server thread panicked");
    assert!(request.to_lowercase().contains("user-agent: mozilla/5.0"));
}

#[test]
fn info_custom_user_agent_is_sent() {
    let (url, handle) = serve_once(PAGE);

    let output = run_cli(&["info", "--user-agent", "webpage-cli-e2e-agent", &url]);

    assert!(output.status.success());
    let request = handle.join().expect("server thread panicked");
    assert!(
        request
            .to_lowercase()
            .contains("user-agent: webpage-cli-e2e-agent")
    );
}

#[test]
fn info_writes_json_to_output_directory() {
    let (url, handle) = serve_once(PAGE);
    let dir = unique_temp_dir("output-dir");

    let output = run_cli(&["info", "--output-dir", dir.to_str().unwrap(), &url]);

    assert!(
        output.status.success(),
        "command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout_json: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout should still be JSON");

    let entries: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("could not read output dir")
        .map(|entry| entry.expect("could not read entry").path())
        .collect();
    assert_eq!(entries.len(), 1, "expected exactly one output file");

    let file_name = entries[0]
        .file_name()
        .expect("output path should have a file name")
        .to_string_lossy()
        .into_owned();
    assert!(
        file_name.starts_with("127-0-0-1-"),
        "unexpected file name: {file_name}"
    );
    assert!(
        file_name.ends_with("-info.json"),
        "unexpected file name: {file_name}"
    );

    let file_json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&entries[0]).expect("could not read output file"))
            .expect("output file should be valid JSON");
    assert_eq!(file_json, stdout_json);

    handle.join().expect("server thread panicked");
    fs::remove_dir_all(&dir).expect("could not remove output dir");
}

#[test]
fn info_verbose_prints_diagnostics_instead_of_json() {
    let (url, handle) = serve_once(PAGE);

    let output = run_cli(&["info", "--verbose", &url]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("user_agent: Mozilla/5.0"),
        "stdout: {stdout}"
    );
    assert!(stdout.contains(&format!("url: {url}")), "stdout: {stdout}");
    assert!(
        !stdout.contains("\"http\""),
        "verbose mode should not print JSON: {stdout}"
    );

    handle.join().expect("server thread panicked");
}

#[test]
fn info_verbose_reports_written_file() {
    let (url, handle) = serve_once(PAGE);
    let dir = unique_temp_dir("verbose-output");

    let output = run_cli(&[
        "info",
        "--verbose",
        "--output-dir",
        dir.to_str().unwrap(),
        &url,
    ]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Output written to file successfully."),
        "stdout: {stdout}"
    );
    assert_eq!(
        fs::read_dir(&dir)
            .expect("could not read output dir")
            .count(),
        1
    );

    handle.join().expect("server thread panicked");
    fs::remove_dir_all(&dir).expect("could not remove output dir");
}

#[test]
fn info_fails_for_unreachable_url() {
    // Binding and immediately dropping a listener gives us a port that is
    // very likely closed, without depending on any external network.
    let listener = TcpListener::bind("127.0.0.1:0").expect("could not bind listener");
    let addr = listener
        .local_addr()
        .expect("could not read listener address");
    drop(listener);

    let output = run_cli(&["info", &format!("http://{addr}")]);

    assert!(!output.status.success());
    assert!(
        !output.stderr.is_empty(),
        "expected an error message on stderr"
    );
}

#[test]
fn info_without_url_exits_with_usage_error() {
    let output = run_cli(&["info"]);

    assert_eq!(output.status.code(), Some(2));
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage"), "stderr: {stderr}");
}

#[test]
fn version_flag_prints_package_version() {
    let output = run_cli(&["--version"]);

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.starts_with(&format!("webpage-cli {}", env!("CARGO_PKG_VERSION"))),
        "stdout: {stdout}"
    );
}
