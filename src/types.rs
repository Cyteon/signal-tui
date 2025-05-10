use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalContactList {
    pub jsonrpc: String,
    pub result: Vec<SignalContact>,
    pub id: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalMessageEvent {
    pub jsonrpc: String,
    pub method: String,
    pub params: SignalMessageEventParams,
}

// not exactly pure from rpc

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalMessageEventParams {
    pub result: SignalMessageEventResult,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageEventResult {
    pub envelope: SignalMessageEventEnvelope,
    pub account: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageEventEnvelope {
    pub source_uuid: String,
    pub source_name: String,
    pub timestamp: u64,
    pub data_message: Option<SignalMessageEventDataMessage>,
    pub sync_message: Option<SignalMessageEventSyncMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageEventDataMessage {
    pub message: Option<String>,
    pub expires_in_seconds: u64,
    pub group_info: Option<SignalMessageEventGroupInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageEventSyncMessage {
    pub sent_message: Option<SignalMessageEventSentMessage>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageEventSentMessage {
    pub destination_uuid: Option<String>,
    pub message: Option<String>,
    pub expires_in_seconds: u64,
    pub group_info: Option<SignalMessageEventGroupInfo>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SignalMessageEventGroupInfo {
  pub group_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignalGroup {
    pub id: String,
    pub name: String,
    pub description: String,
    pub is_member: bool,
    pub is_blocked: bool,
    pub members: Vec<SignalUser>,
    pub pending_members: Vec<SignalUser>,
    pub requesting_members: Vec<SignalUser>,
    pub admins: Vec<SignalUser>,
    pub group_invite_link: String
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignalContact {
    pub number: Option<String>,
    pub uuid: String,
    pub username: Option<String>,
    pub name: String,
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub nick_name: Option<String>,
    pub nick_given_name: Option<String>,
    pub nick_family_name: Option<String>,
    pub note: Option<String>,
    pub color: Option<String>,
    pub is_hidden: bool,
    pub is_blocked: bool,
    pub message_expiration_time: u64,
    pub profile: Option<SignalProfile>
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct SignalProfile {
    pub given_name: Option<String>,
    pub family_name: Option<String>,
    pub about: Option<String>,
    pub about_emoji: Option<String>,
    pub has_avatar: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignalUser {
    pub number: Option<String>,
    pub uuid: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct SignalAccount {
    pub number: String,
}