use std::io::{self, Write, Read};
use hostname::get;
use std::process::{Command, Stdio};

use crate::types::{self, SignalAccount};

pub fn create_cli(args: String) -> io::Result<std::process::Child> {
    let child = Command::new("./signal-cli")
        .args(args.split_whitespace())
        .arg("jsonRpc")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;

    Ok(child)
}

pub fn list_accounts(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout) -> Vec<SignalAccount> {
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"listAccounts\",\"params\":{{}},\"id\":\"1\"}}").unwrap();

    let response = read_res(stdin, stdout);
    let data: types::SignalAccountList = serde_json::from_str(&response).unwrap();

    data.result
}

pub fn read_res(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout) -> String {
    let mut string = String::new();
    stdout.read_to_string(&mut string).unwrap();

    string
}

pub fn link_device(stdin: &mut std::process::ChildStdin, stdout: &mut std::process::ChildStdout) {
    writeln!(stdin, "{{\"jsonrpc\":\"2.0\",\"method\":\"startLink\",\"id\":\"5\"}}").unwrap();

    let response = read_res(stdin, stdout);
    let data: types::SignalLinkingResponse = serde_json::from_str(&response).unwrap();
    let link = data.result.get("deviceLinkUri").unwrap().clone();

    finish_link(stdin, stdout, link);
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

    writeln!(
        stdin, 
        r#"
            {{
                "jsonrpc":"2.0",
                "method":"finishLink",
                "id":"6",
                "params": {{
                    "deviceLinkUri": {},
                    "deviceName": {}
                }}
            }}
        "#,
        link, name
    ).unwrap();
}