use async_tungstenite::tungstenite::{handshake, Message};
use chat_core::{Error, Result};
use futures::channel::oneshot::Sender;
use serde::{Deserialize, Serialize};

#[derive(Debug)]
pub enum ProtocolFormat {
    Json,
    Msgpack,
}

#[derive(Debug)]
pub struct ProtocolConfiguration {
    protocol_version: i32,
    format: ProtocolFormat,
    session_token: Option<String>,
}

impl ProtocolConfiguration {
    pub fn from(
        protocol_version: i32,
        format: ProtocolFormat,
        session_token: Option<String>,
    ) -> Self {
        Self {
            protocol_version,
            format,
            session_token,
        }
    }

    pub fn decode<'a, T: Deserialize<'a>>(&self, msg: &'a Message) -> Result<T> {
        match self.format {
            ProtocolFormat::Json => {
                if let Message::Text(text) = msg {
                    serde_json::from_str(text).map_err(|_| Error::InternalError)
                } else {
                    Err(Error::InternalError)
                }
            }
            ProtocolFormat::Msgpack => {
                if let Message::Binary(buf) = msg {
                    rmp_serde::from_slice(buf).map_err(|_| Error::InternalError)
                } else {
                    Err(Error::InternalError)
                }
            }
        }
    }

    pub fn encode<T: Serialize>(&self, data: &T) -> Message {
        match self.format {
            ProtocolFormat::Json => {
                Message::Text(serde_json::to_string(data).expect("Failed to serialize (as json)."))
            }
            ProtocolFormat::Msgpack => Message::Binary(
                rmp_serde::to_vec_named(data).expect("Failed to serialize (as msgpack)."),
            ),
        }
    }

    pub fn set_session_token(&mut self, token: String) {
        self.session_token.replace(token);
    }

    pub fn get_session_token(&self) -> &Option<String> {
        &self.session_token
    }

    pub fn get_protocol_version(&self) -> i32 {
        self.protocol_version
    }

    pub fn get_protocol_format(&self) -> &ProtocolFormat {
        &self.format
    }
}

pub struct WebsocketHandshakeCallback {
    sender: Sender<ProtocolConfiguration>,
}

impl WebsocketHandshakeCallback {
    pub fn from(sender: Sender<ProtocolConfiguration>) -> Self {
        Self { sender }
    }
}

impl handshake::server::Callback for WebsocketHandshakeCallback {
    fn on_request(
        self,
        request: &handshake::server::Request,
        response: handshake::server::Response,
    ) -> Result<handshake::server::Response, handshake::server::ErrorResponse> {
        let query = request.uri().query().unwrap_or_default();
        let params = querystring::querify(query);

        let mut protocol_version = 1;
        let mut format = ProtocolFormat::Json;
        let mut session_token = None;

        for (key, value) in params {
            match key {
                "version" => {
                    if let Ok(version) = value.parse() {
                        protocol_version = version;
                    }
                }
                "format" => match value {
                    "json" => format = ProtocolFormat::Json,
                    "msgpack" => format = ProtocolFormat::Msgpack,
                    _ => {}
                },
                "token" => session_token = Some(value.into()),
                _ => {}
            }
        }

        if self
            .sender
            .send(ProtocolConfiguration {
                protocol_version,
                format,
                session_token,
            })
            .is_ok()
        {
            Ok(response)
        } else {
            Err(handshake::server::ErrorResponse::new(None))
        }
    }
}
