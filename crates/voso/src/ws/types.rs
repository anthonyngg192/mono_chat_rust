use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use strum::IntoStaticStr;
use warp::ws::Message;

use mediasoup::prelude::*;
use mediasoup::rtp_parameters::{MediaKind, RtpCapabilitiesFinalized, RtpParameters};

use crate::rtc::types::{ConnectTransportData, InitializationInput, TransportInitData};
use crate::state::user::{ProduceType, UserInfo};

#[derive(Deserialize)]
pub struct WSCommand {
    pub id: Option<u64>,
    #[serde(flatten)]
    pub command_type: WSCommandType,
}

#[derive(Deserialize, IntoStaticStr, Clone)]
#[serde(tag = "type", content = "data")]
pub enum WSCommandType {
    #[serde(rename_all = "camelCase")]
    Authenticate {
        room_id: String,
        token: String,
    },

    InitializeTransports {
        #[serde(flatten)]
        init_data: InitializationInput,
    },
    ConnectTransport {
        #[serde(flatten)]
        connect_data: ConnectTransportData,
    },

    RoomInfo,

    #[serde(rename_all = "camelCase")]
    StartProduce {
        #[serde(rename = "type")]
        produce_type: ProduceType,
        rtp_parameters: RtpParameters,
    },
    #[serde(rename_all = "camelCase")]
    StopProduce {
        #[serde(rename = "type")]
        produce_type: ProduceType,
    },

    #[serde(rename_all = "camelCase")]
    StartConsume {
        #[serde(rename = "type")]
        produce_type: ProduceType,
        user_id: String,
    },
    StopConsume {
        /// Consumer ID
        id: ConsumerId,
    },
    SetConsumerPause {
        /// Consumer ID
        id: ConsumerId,
        paused: bool,
    },
}

impl WSReplyType {
    pub fn get_message(self, command_id: Option<u64>) -> Result<Message, serde_json::Error> {
        let reply = WSReply {
            id: command_id,
            reply_type: self,
        };

        Ok(Message::text(serde_json::to_string(&reply)?))
    }
}

#[derive(Serialize)]
pub struct WSReply {
    pub id: Option<u64>,
    #[serde(flatten)]
    pub reply_type: WSReplyType,
}

#[derive(Serialize, Deserialize)]
pub enum WSReplyType {
    Authenticate {
        vortex_version: &'static str,
        user_id: String,
        room_id: String,
        rtp_capabilities: RtpCapabilitiesFinalized,
    },

    InitializeTransports {
        reply_data: Box<TransportInitData>,
    },
    ConnectTransport,

    RoomInfo {
        id: String,
        video_allowed: bool,
        users: HashMap<String, UserInfo>,
    },

    StartProduce {
        producer_id: ProducerId,
    },
    StopProduce,

    StartConsume {
        id: ConsumerId,
        producer_id: ProducerId,
        kind: MediaKind,
        rtp_parameters: RtpParameters,
    },
    StopConsume,
    SetConsumerPause,
}

#[derive(Serialize, Deserialize)]
pub enum WSEvent {
    Joined {
        id: String,
    },
    Left {
        id: String,
    },

    StartProduce {
        id: String,
        #[serde(rename = "type")]
        produce_type: ProduceType,
    },
    UserStopProduce {
        id: String,
        #[serde(rename = "type")]
        produce_type: ProduceType,
    },
}
