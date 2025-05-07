use std::collections::HashMap;

use serde::{Deserialize, Serialize};

// {"jsonrpc":"2.0","result":{"deviceLinkUri":"sgnl://linkdevice?uuid=X&pub_key=X"},"id":"5"}
#[derive(Serialize, Deserialize, Debug)]
pub struct SignalLinkingResponse {
    pub jsonrpc: String,
    pub result: HashMap<String, String>,
    pub id: String
}

// { "jsonrpc": "2.0", "method": "listAccounts", "params": {}, "id": "1" }
#[derive(Serialize, Deserialize, Debug)]
pub struct SignalAccountList {
    pub jsonrpc: String,
    pub result: Vec<SignalAccount>,
    pub id: String
}

// not rpc
#[derive(Serialize, Deserialize, Debug)]
pub struct SignalAccount {
    pub uuid: String,
    pub number: String,
}