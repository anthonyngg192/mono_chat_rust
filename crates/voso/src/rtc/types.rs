use serde::{Deserialize, Serialize};
use std::net::IpAddr;

use mediasoup::data_structures::TransportProtocol;
use mediasoup::prelude::*;
use mediasoup::sctp_parameters::SctpParameters;
use mediasoup::srtp_parameters::SrtpParameters;

#[derive(Serialize, Deserialize, Clone)]
pub struct InitializationInput {
    pub(super) rtp_capabilities: RtpCapabilities,

    #[serde(flatten)]
    pub(super) mode: InitializationInputMode,
}

#[derive(Serialize, Deserialize, Clone)]
#[serde(tag = "mode")]
pub enum InitializationInputMode {
    #[serde(rename = "SplitWebRTC")]
    SplitWebRtc,

    #[serde(rename = "CombinedWebRTC")]
    CombinedWebRtc,

    #[serde(rename = "CombinedRTP")]
    CombinedRtp,
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum TransportInitData {
    SplitWebRtc {
        send_transport: WebRtcTransportInitData,
        recv_transport: WebRtcTransportInitData,
    },
    CombinedWebRtc {
        transport: WebRtcTransportInitData,
    },
    CombinedRtp {
        ip: IpAddr,
        port: u16,
        protocol: TransportProtocol,
        id: TransportId,
        srtp_crypto_suite: SrtpCryptoSuite,
    },
}

#[derive(Serialize, Deserialize)]
pub struct WebRtcTransportInitData {
    pub id: TransportId,
    pub ice_parameters: IceParameters,
    pub ice_candidates: Vec<IceCandidate>,
    pub dtls_parameters: DtlsParameters,
    pub sctp_parameters: Option<SctpParameters>,
}

#[derive(Deserialize, Clone)]
pub struct ConnectTransportData {
    pub id: TransportId,

    #[serde(flatten)]
    pub params: ConnectTransportParams,
}

#[derive(Deserialize, Clone)]
#[serde(untagged)]
#[serde(rename_all = "camelCase")]
pub enum ConnectTransportParams {
    #[serde(rename_all = "camelCase")]
    WebRtc { dtls_parameters: DtlsParameters },

    #[serde(rename_all = "camelCase")]
    Rtp { srtp_parameters: SrtpParameters },
}
