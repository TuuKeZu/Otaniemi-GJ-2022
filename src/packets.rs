use crate::game::{Card, Color, GameStatistics};
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, strum_macros::Display)]
#[serde(tag = "type", content = "data")]
pub enum PacketType {
    Register(String),                                     // username
    GameData(Uuid, String, Vec<(Uuid, String)>), // self_id, self_username, Vec<(id, username)>
    Connect(Uuid, String),                       // id, username
    Disconnect(Uuid, String),                    // id, username
    Message(String, String),                     // content
    Error(u64, String),                                        // error-code, body
}
