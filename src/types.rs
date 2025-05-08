use std::collections::HashMap;
use serde::{Deserialize, Serialize};

// -- app --

#[derive(Debug, Default, Clone, Copy)]
pub struct App {
    pub state: AppState,
    pub download_status: u16,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum AppState {
    #[default]
    Running,
    Started,
    Quitting,
}

// -- rpc --

// {"jsonrpc":"2.0","result":{"deviceLinkUri":"sgnl://linkdevice?uuid=X&pub_key=X"},"id":"5"}
#[derive(Serialize, Deserialize, Debug)]
pub struct SignalLinkingResponse {
    pub jsonrpc: String,
    pub result: HashMap<String, String>,
    pub id: String
}

// from { "jsonrpc": "2.0", "method": "listAccounts", "params": {}, "id": "1" }
#[derive(Serialize, Deserialize, Debug)]
pub struct SignalAccountList {
    pub jsonrpc: String,
    pub result: Vec<SignalAccount>,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalAccount {
    pub number: String,
}