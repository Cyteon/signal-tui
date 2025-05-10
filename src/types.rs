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

/*{
  "jsonrpc": "2.0",
  "result": [
    {
      "id": "Pmpi+EfPWmsxiomLe9Nx2XF9HOE483p6iKiFj65iMwI=",
      "name": "My Group",
      "description": "It’s special because it is mine.",
      "isMember": true,
      "isBlocked": false,
      "members": [
        "+33123456789",
        "+440123456789"
      ],
      "pendingMembers": [],
      "requestingMembers": [],
      "admins": [
        "+33123456789",
        "+440123456789"
      ],
      "groupInviteLink": "https://signal.group/#CjQKIAtcbUw482i7bqvmJCwdgvg0FMif52N5v9lGg_bE4U3zEhCjHKSaPzWImMpnCbU8A1r0"
    }
  ],
  "id": "my special mark"
}*/
#[derive(Serialize, Deserialize, Debug)]
pub struct SignalGroupList {
    pub jsonrpc: String,
    pub result: Vec<SignalGroup>,
    pub id: String
}

// not exactly pure from rpc

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_member: bool,
    pub is_blocked: bool,
    pub members: Vec<String>,
    pub pending_members: Vec<String>,
    pub requesting_members: Vec<String>,
    pub admins: Vec<String>,
    pub group_invite_link: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalAccount {
    pub number: String,
}