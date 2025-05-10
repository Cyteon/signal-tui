use std::{io::{self, Read, Write}, path::PathBuf};
use color_eyre::owo_colors::OwoColorize;
use hostname::get;
use random_string::generate;
use ratatui::{layout::{Alignment, Constraint, Direction, Layout}, style::{Color, Style}, widgets::{Block, Borders, Gauge, Paragraph}, DefaultTerminal};
use std::process::{Command, Stdio};
use reqwest::blocking::Client;
use flate2::read::GzDecoder;
use tar::Archive;

use crate::types::{self, SignalAccount, SignalContact, SignalGroup};

pub fn create_cli(path: PathBuf, args: String) -> io::Result<std::process::Child> {
    let cli_path = match std::env::consts::OS {
        "windows" => path.join("signal-cli/bin/signal-cli.bat"),
        _ => path.join("signal-cli/bin/signal-cli")
    };

    let child = Command::new(cli_path)
        .args(args.split_whitespace())
        .arg("jsonRpc")
        .arg("--receive-mode=manual")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    Ok(child)
}

pub fn list_accounts(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout) -> Vec<SignalAccount> {
    let id = generate_id();
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"listAccounts\",\"params\":{{}},\"id\":\"{}\"}}", id).unwrap();

    let mut retry_attempts_until_error = 20;
    let mut response= String::new();

    while response.is_empty() || !response.contains(&id) {
        response = read_res(stdout);

        retry_attempts_until_error -= 1;

        if retry_attempts_until_error == 0 {
            // i have no idea if this is a good idea buttttttt
            panic!("Failed to get response from signal-cli\nThis means that signal-cli most likely crashed\nPlease ensure you have java installed as that is a requirement");
        }

        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let data: types::SignalAccountList = serde_json::from_str(&response).unwrap();
    data.result.iter()
        .map(|account| SignalAccount {
            number: account.number.clone(),
        })
        .collect()
}

pub fn read_res(stdout: &mut std::process::ChildStdout) -> String {
    use std::io::{BufRead, BufReader};
    
    let mut reader = BufReader::new(stdout);
    let mut line = String::new();
    
    match reader.read_line(&mut line) {
        Ok(_) => line,
        Err(e) => {
            eprintln!("Error reading response: {}", e);
            String::new()
        }
    }
}

pub fn link_device(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout) -> String {
    let id = generate_id();
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"startLink\",\"id\":\"{}\"}}", id).unwrap();

    let mut response = String::new();

    while response.is_empty() || !response.contains(&id) {
        response = read_res(stdout);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let data: types::SignalLinkingResponse = serde_json::from_str(&response).unwrap();
    let link = data.result.get("deviceLinkUri").unwrap().clone();

    link
}

pub fn finish_link(
    stdin: &mut std::process::ChildStdin, 
    stdout: &mut std::process::ChildStdout, 
    link: String
) {
    let name = match get() {
        Ok(name) => name.into_string().unwrap_or("Unknown".to_string()),
        Err(_) => "Unknown".to_string()
    };

    let id = generate_id();

    let mut content = format!(
        r#"
            {{
                "jsonrpc":"2.0",
                "method":"finishLink",
                "id":"{}",
                "params": {{
                    "deviceLinkUri": "{}",
                    "deviceName": "{}"
                }}
            }}
        "#,
        id, link, name
    );

    content = content.replace("\n", "");
    content = content.replace("\t", "");
    content = content.trim().to_string();

    writeln!(
        stdin, "{}", content
    ).unwrap();

    let mut response = String::new();

    // it should have id:6
    while response.is_empty() || !response.contains(&id) {
        response = read_res(stdout);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

pub fn generate_id() -> String {
    let charset = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    generate(32, charset)
}

pub fn sync(
    stdin: &mut std::process::ChildStdin, 
    stdout: &mut std::process::ChildStdout,
    db: &rusqlite::Connection,
    account_number: String
) -> (Vec<SignalGroup>, Vec<SignalContact>) {
    // groups

    let mut id = generate_id();
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"listGroups\",\"id\":\"{}\"}}", id).unwrap();

    let mut response = String::new();

    while response.is_empty() || !response.contains(&id) {
        response = read_res(stdout);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let data: types::SignalGroupList = serde_json::from_str(&response).unwrap();

    let groups = data.result;

    // contacts

    id = generate_id();
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"listContacts\",\"id\":\"{}\"}}", id).unwrap();

    response = String::new();

    while response.is_empty() || !response.contains(&id) {
        response = read_res(stdout);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }

    let data: types::SignalContactList = serde_json::from_str(&response).unwrap();
    let contacts = data.result;

    (groups, contacts)
}

// cli download

pub fn download_cli(terminal: &mut DefaultTerminal, path: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://github.com/AsamK/signal-cli/releases/download/v0.13.14/signal-cli-0.13.14.tar.gz";
    let download_path = path.join("signal-cli-0.13.14.tar.gz");
    let mut file = std::fs::File::create(&download_path)?;

    let client = Client::new();
    let mut response = client.get(url).send()?;

    let total_size = response.content_length().unwrap_or(0);

    let mut downloaded: u64 = 0;
    let mut buffer = [0; 8192];

    loop {
        let b = response.read(&mut buffer)?;

        if b == 0 {
            let tar_gz = GzDecoder::new(std::fs::File::open(&download_path)?);
            let mut archive = Archive::new(tar_gz);
            archive.unpack(&path)?;
            std::fs::remove_file(&download_path)?;

            std::fs::rename(
                path.join("signal-cli-0.13.14"),
                path.join("signal-cli")
            )?;

            break;
        }

        file.write_all(&buffer[..b])?;
        downloaded += b as u64;

        let percent = (downloaded as f64 / total_size as f64 * 100.0) as u16;

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(5)
                .constraints(
                    [
                        Constraint::Length(1), // Title line
                        Constraint::Length(3), // Gauge
                        Constraint::Min(0),    // Spacer
                    ]
                    .as_ref(),
                )
                .split(f.area());
        
            let title = Paragraph::new("Downloading signal-cli...")
                .alignment(Alignment::Center);
        
            f.render_widget(title, chunks[0]);
            
            let gauge = Gauge::default()
                .block(Block::default().borders(Borders::ALL))
                .gauge_style(Style::default().fg(Color::Green))
                .label(format!("{}%", percent))
                .percent(percent);

            f.render_widget(gauge, chunks[1]);
        })?;
    }

    Ok(())
}