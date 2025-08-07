use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::ReplyTarget;
use crate::util::{join_with_space, split_spaces};
use crate::{
    FinalParam, MedialParam, Message, ParameterList, ParameterListSizeError, RawMessage,
    TryFromStringError, Verb,
};
use std::fmt::{self, Write};
use thiserror::Error;

// As of 2025-07-07, all CAP messages (as per
// <https://ircv3.net/specs/extensions/capability-negotiation.html>) take one
// of the following formats:
//
// - Client-to-server:
//     - `CAP <subcommand>`
//     - `CAP <subcommand> <parameter>`
//
// - Server-to-client:
//     - `CAP <nick-or-star> <subcommand> <parameter>`
//     - `CAP <nick-or-star> <subcommand> * <parameter>`
//
// As long as it stays this way — with client-to-server messages having at most
// one subcommand parameter and server-to-client messages having at least one
// subcommand parameter — we can reliably determine whether a message being
// parsed has a `<nick-or-star>` parameter.

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Cap {
    LsRequest(CapLsRequest),
    LsResponse(CapLsResponse),
    ListRequest(CapListRequest),
    ListResponse(CapListResponse),
    Req(CapReq),
    Ack(CapAck),
    Nak(CapNak),
    End(CapEnd),
    New(CapNew),
    Del(CapDel),
}

impl ClientMessageParts for Cap {
    fn into_parts(self) -> (Verb, ParameterList) {
        match self {
            Cap::LsRequest(c) => c.into_parts(),
            Cap::LsResponse(c) => c.into_parts(),
            Cap::ListRequest(c) => c.into_parts(),
            Cap::ListResponse(c) => c.into_parts(),
            Cap::Req(c) => c.into_parts(),
            Cap::Ack(c) => c.into_parts(),
            Cap::Nak(c) => c.into_parts(),
            Cap::End(c) => c.into_parts(),
            Cap::New(c) => c.into_parts(),
            Cap::Del(c) => c.into_parts(),
        }
    }

    fn to_irc_line(&self) -> String {
        match self {
            Cap::LsRequest(c) => c.to_irc_line(),
            Cap::LsResponse(c) => c.to_irc_line(),
            Cap::ListRequest(c) => c.to_irc_line(),
            Cap::ListResponse(c) => c.to_irc_line(),
            Cap::Req(c) => c.to_irc_line(),
            Cap::Ack(c) => c.to_irc_line(),
            Cap::Nak(c) => c.to_irc_line(),
            Cap::End(c) => c.to_irc_line(),
            Cap::New(c) => c.to_irc_line(),
            Cap::Del(c) => c.to_irc_line(),
        }
    }
}

impl From<Cap> for Message {
    fn from(value: Cap) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Cap> for RawMessage {
    fn from(value: Cap) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Cap {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Cap, ClientMessageError> {
        match params.len() {
            1 => {
                // CAP <subcommand>
                let Ok((subcmd,)): Result<(FinalParam,), _> = params.try_into() else {
                    unreachable!("ParameterList should be convertible to 1-tuple when len is 1");
                };
                match subcmd.to_ascii_uppercase().as_str() {
                    "LS" => Ok(Cap::from(CapLsRequest::new())),
                    "LIST" => Ok(Cap::from(CapListRequest)),
                    "END" => Ok(Cap::from(CapEnd)),
                    "REQ" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 2,
                            received: 1,
                        },
                    )),
                    "ACK" | "NAK" | "NEW" | "DEL" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 3,
                            received: 1,
                        },
                    )),
                    _ => Err(ClientMessageError::UnknownCap(subcmd.into_inner())),
                }
            }
            2 => {
                // CAP <subcommand> <parameter>
                let Ok((subcommand, parameter)): Result<(MedialParam, FinalParam), _> =
                    params.try_into()
                else {
                    unreachable!("ParameterList should be convertible to 2-tuple when len is 2");
                };
                match subcommand.to_ascii_uppercase().as_str() {
                    "LS" => match parameter.as_str().parse::<u32>() {
                        Ok(version) => Ok(Cap::from(CapLsRequest::new_with_version(version))),
                        Err(inner) => Err(ClientMessageError::Int {
                            string: parameter.into_inner(),
                            inner,
                        }),
                    },
                    "REQ" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| CapabilityRequest::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapReq { capabilities }))
                    }
                    "LIST" | "END" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 1,
                            received: 2,
                        },
                    )),
                    "ACK" | "NAK" | "NEW" | "DEL" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 3,
                            received: 1,
                        },
                    )),
                    _ => Err(ClientMessageError::UnknownCap(subcommand.into_inner())),
                }
            }
            3 => {
                // CAP <nick-or-star> <subcommand> <parameter>
                let Ok((target, subcommand, parameter)): Result<
                    (MedialParam, MedialParam, FinalParam),
                    _,
                > = params.try_into() else {
                    unreachable!("ParameterList should be convertible to 3-tuple when len is 3");
                };
                let target = ReplyTarget::try_from(target.into_inner())?;
                match subcommand.to_ascii_uppercase().as_str() {
                    "LS" => {
                        let mut capabilities = Vec::new();
                        for word in split_spaces(parameter.as_str()) {
                            if let Some((key, value)) = word.split_once('=') {
                                let key = Capability::try_from(key.to_owned())?;
                                let value = CapabilityValue::try_from(value.to_owned())?;
                                capabilities.push((key, Some(value)));
                            } else {
                                capabilities.push((Capability::try_from(word.to_owned())?, None));
                            }
                        }
                        Ok(Cap::from(CapLsResponse {
                            target,
                            continued: false,
                            capabilities,
                        }))
                    }
                    "LIST" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| Capability::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapListResponse {
                            target,
                            continued: false,
                            capabilities,
                        }))
                    }
                    "ACK" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| CapabilityRequest::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapAck {
                            target,
                            capabilities,
                        }))
                    }
                    "NAK" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| Capability::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapNak {
                            target,
                            capabilities,
                        }))
                    }
                    "NEW" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| Capability::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapNew {
                            target,
                            capabilities,
                        }))
                    }
                    "DEL" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| Capability::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapDel {
                            target,
                            capabilities,
                        }))
                    }
                    "END" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 1,
                            received: 2,
                        },
                    )),
                    "REQ" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 2,
                            received: 1,
                        },
                    )),
                    _ => Err(ClientMessageError::UnknownCap(subcommand.into_inner())),
                }
            }
            4 => {
                // CAP <nick-or-star> <subcommand> * <parameter>
                let Ok((target, subcommand, star, parameter)): Result<
                    (MedialParam, MedialParam, MedialParam, FinalParam),
                    _,
                > = params.try_into() else {
                    unreachable!("ParameterList should be convertible to 4-tuple when len is 4");
                };
                if star != "*" {
                    return Err(ClientMessageError::ParamValue {
                        got: star.into_inner(),
                        expected: "*",
                    });
                }
                let target = ReplyTarget::try_from(target.into_inner())?;
                match subcommand.to_ascii_uppercase().as_str() {
                    "LS" => {
                        let mut capabilities = Vec::new();
                        for word in split_spaces(parameter.as_str()) {
                            if let Some((key, value)) = word.split_once('=') {
                                let key = Capability::try_from(key.to_owned())?;
                                let value = CapabilityValue::try_from(value.to_owned())?;
                                capabilities.push((key, Some(value)));
                            } else {
                                capabilities.push((Capability::try_from(word.to_owned())?, None));
                            }
                        }
                        Ok(Cap::from(CapLsResponse {
                            target,
                            continued: true,
                            capabilities,
                        }))
                    }
                    "LIST" => {
                        let capabilities = split_spaces(parameter.as_str())
                            .map(|s| Capability::try_from(s.to_owned()))
                            .collect::<Result<Vec<_>, _>>()?;
                        Ok(Cap::from(CapListResponse {
                            target,
                            continued: true,
                            capabilities,
                        }))
                    }
                    "END" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 1,
                            received: 2,
                        },
                    )),
                    "REQ" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 2,
                            received: 1,
                        },
                    )),
                    "ACK" | "NAK" | "NEW" | "DEL" => Err(ClientMessageError::ParamQty(
                        ParameterListSizeError::Exact {
                            required: 3,
                            received: 1,
                        },
                    )),
                    _ => Err(ClientMessageError::UnknownCap(subcommand.into_inner())),
                }
            }
            len => Err(ClientMessageError::ParamQty(
                ParameterListSizeError::Range {
                    min_required: 1,
                    max_required: 4,
                    received: len,
                },
            )),
        }
    }
}

#[derive(Clone, Debug, Default, Eq, Hash, PartialEq)]
pub struct CapLsRequest {
    pub version: Option<u32>,
}

impl CapLsRequest {
    pub fn new() -> CapLsRequest {
        CapLsRequest { version: None }
    }

    pub fn new_with_version(version: u32) -> CapLsRequest {
        CapLsRequest {
            version: Some(version),
        }
    }
}

impl ClientMessageParts for CapLsRequest {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder();
        let Ok(subcmd) = "LS".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        if let Some(v) = self.version {
            let Ok(vs) = MedialParam::try_from(v.to_string()) else {
                unreachable!();
            };
            builder.push_medial(vs);
        }
        (Verb::Cap, builder.finish())
    }

    fn to_irc_line(&self) -> String {
        let mut s = String::from("CAP LS");
        if let Some(v) = self.version {
            let _ = write!(&mut s, " {v}");
        }
        s
    }
}

impl From<CapLsRequest> for Cap {
    fn from(value: CapLsRequest) -> Cap {
        Cap::LsRequest(value)
    }
}

impl From<CapLsRequest> for ClientMessage {
    fn from(value: CapLsRequest) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapLsRequest> for Message {
    fn from(value: CapLsRequest) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapLsRequest> for RawMessage {
    fn from(value: CapLsRequest) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapLsResponse {
    pub target: ReplyTarget,
    // Whether there's an asterisk parameter between the subcommand and the
    // actual parameter:
    pub continued: bool,
    pub capabilities: Vec<(Capability, Option<CapabilityValue>)>,
}

impl CapLsResponse {
    fn final_param(&self) -> FinalParam {
        let mut s = String::new();
        let mut first = true;
        for (cap, value) in &self.capabilities {
            if !std::mem::replace(&mut first, false) {
                s.push(' ');
            }
            s.push_str(cap.as_str());
            if let Some(v) = value.as_ref() {
                s.push('=');
                s.push_str(v.as_str());
            }
        }
        let Ok(p) = FinalParam::try_from(s) else {
            unreachable!();
        };
        p
    }
}

impl ClientMessageParts for CapLsResponse {
    fn into_parts(self) -> (Verb, ParameterList) {
        let finalp = self.final_param();
        let mut builder = ParameterList::builder().with_medial(self.target);
        let Ok(subcmd) = "LS".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        if self.continued {
            let Ok(star) = "*".parse::<MedialParam>() else {
                unreachable!();
            };
            builder.push_medial(star);
        }
        let params = builder.with_final(finalp);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        let mut s = format!("CAP {} LS ", self.target);
        if self.continued {
            s.push_str("* ");
        }
        let _ = write!(&mut s, ":{}", self.final_param());
        s
    }
}

impl From<CapLsResponse> for Cap {
    fn from(value: CapLsResponse) -> Cap {
        Cap::LsResponse(value)
    }
}

impl From<CapLsResponse> for ClientMessage {
    fn from(value: CapLsResponse) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapLsResponse> for Message {
    fn from(value: CapLsResponse) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapLsResponse> for RawMessage {
    fn from(value: CapLsResponse) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapListRequest;

impl ClientMessageParts for CapListRequest {
    fn into_parts(self) -> (Verb, ParameterList) {
        let Ok(subcmd) = "LIST".parse::<MedialParam>() else {
            unreachable!()
        };
        (
            Verb::Cap,
            ParameterList::builder().with_medial(subcmd).finish(),
        )
    }

    fn to_irc_line(&self) -> String {
        String::from("CAP LIST")
    }
}

impl From<CapListRequest> for Cap {
    fn from(value: CapListRequest) -> Cap {
        Cap::ListRequest(value)
    }
}

impl From<CapListRequest> for ClientMessage {
    fn from(value: CapListRequest) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapListRequest> for Message {
    fn from(value: CapListRequest) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapListRequest> for RawMessage {
    fn from(value: CapListRequest) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapListResponse {
    pub target: ReplyTarget,
    // Whether there's an asterisk parameter between the subcommand and the
    // actual parameter:
    pub continued: bool,
    pub capabilities: Vec<Capability>,
}

impl ClientMessageParts for CapListResponse {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder().with_medial(self.target);
        let Ok(subcmd) = "LIST".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        if self.continued {
            let Ok(star) = "*".parse::<MedialParam>() else {
                unreachable!();
            };
            builder.push_medial(star);
        }
        let Ok(finalp) = FinalParam::try_from(join_with_space(&self.capabilities).to_string())
        else {
            unreachable!()
        };
        let params = builder.with_final(finalp);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        let mut s = format!("CAP {} LIST ", self.target);
        if self.continued {
            s.push_str("* ");
        }
        let _ = write!(&mut s, ":{}", join_with_space(&self.capabilities));
        s
    }
}

impl From<CapListResponse> for Cap {
    fn from(value: CapListResponse) -> Cap {
        Cap::ListResponse(value)
    }
}

impl From<CapListResponse> for ClientMessage {
    fn from(value: CapListResponse) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapListResponse> for Message {
    fn from(value: CapListResponse) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapListResponse> for RawMessage {
    fn from(value: CapListResponse) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapReq {
    pub capabilities: Vec<CapabilityRequest>,
}

impl ClientMessageParts for CapReq {
    fn into_parts(self) -> (Verb, ParameterList) {
        let Ok(subcmd) = "REQ".parse::<MedialParam>() else {
            unreachable!()
        };
        let Ok(finalp) = FinalParam::try_from(join_with_space(&self.capabilities).to_string())
        else {
            unreachable!();
        };
        (
            Verb::Cap,
            ParameterList::builder()
                .with_medial(subcmd)
                .with_final(finalp),
        )
    }

    fn to_irc_line(&self) -> String {
        format!("CAP REQ :{}", join_with_space(&self.capabilities))
    }
}

impl From<CapReq> for Cap {
    fn from(value: CapReq) -> Cap {
        Cap::Req(value)
    }
}

impl From<CapReq> for ClientMessage {
    fn from(value: CapReq) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapReq> for Message {
    fn from(value: CapReq) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapReq> for RawMessage {
    fn from(value: CapReq) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapAck {
    pub target: ReplyTarget,
    pub capabilities: Vec<CapabilityRequest>,
}

impl ClientMessageParts for CapAck {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder().with_medial(self.target);
        let Ok(subcmd) = "ACK".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        let Ok(finalp) = FinalParam::try_from(join_with_space(&self.capabilities).to_string())
        else {
            unreachable!()
        };
        let params = builder.with_final(finalp);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "CAP {} ACK :{}",
            self.target,
            join_with_space(&self.capabilities)
        )
    }
}

impl From<CapAck> for Cap {
    fn from(value: CapAck) -> Cap {
        Cap::Ack(value)
    }
}

impl From<CapAck> for ClientMessage {
    fn from(value: CapAck) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapAck> for Message {
    fn from(value: CapAck) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapAck> for RawMessage {
    fn from(value: CapAck) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapNak {
    pub target: ReplyTarget,
    pub capabilities: Vec<Capability>,
}

impl ClientMessageParts for CapNak {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder().with_medial(self.target);
        let Ok(subcmd) = "NAK".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        let Ok(finalp) = FinalParam::try_from(join_with_space(&self.capabilities).to_string())
        else {
            unreachable!()
        };
        let params = builder.with_final(finalp);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "CAP {} NAK :{}",
            self.target,
            join_with_space(&self.capabilities)
        )
    }
}

impl From<CapNak> for Cap {
    fn from(value: CapNak) -> Cap {
        Cap::Nak(value)
    }
}

impl From<CapNak> for ClientMessage {
    fn from(value: CapNak) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapNak> for Message {
    fn from(value: CapNak) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapNak> for RawMessage {
    fn from(value: CapNak) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapEnd;

impl ClientMessageParts for CapEnd {
    fn into_parts(self) -> (Verb, ParameterList) {
        let Ok(subcmd) = "END".parse::<MedialParam>() else {
            unreachable!()
        };
        (
            Verb::Cap,
            ParameterList::builder().with_medial(subcmd).finish(),
        )
    }

    fn to_irc_line(&self) -> String {
        String::from("CAP END")
    }
}

impl From<CapEnd> for Cap {
    fn from(value: CapEnd) -> Cap {
        Cap::End(value)
    }
}

impl From<CapEnd> for ClientMessage {
    fn from(value: CapEnd) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapEnd> for Message {
    fn from(value: CapEnd) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapEnd> for RawMessage {
    fn from(value: CapEnd) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapNew {
    pub target: ReplyTarget,
    pub capabilities: Vec<Capability>,
}

impl ClientMessageParts for CapNew {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder().with_medial(self.target);
        let Ok(subcmd) = "NEW".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        let Ok(finalp) = FinalParam::try_from(join_with_space(&self.capabilities).to_string())
        else {
            unreachable!()
        };
        let params = builder.with_final(finalp);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "CAP {} NEW :{}",
            self.target,
            join_with_space(&self.capabilities)
        )
    }
}

impl From<CapNew> for Cap {
    fn from(value: CapNew) -> Cap {
        Cap::New(value)
    }
}

impl From<CapNew> for ClientMessage {
    fn from(value: CapNew) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapNew> for Message {
    fn from(value: CapNew) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapNew> for RawMessage {
    fn from(value: CapNew) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapDel {
    pub target: ReplyTarget,
    pub capabilities: Vec<Capability>,
}

impl ClientMessageParts for CapDel {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder().with_medial(self.target);
        let Ok(subcmd) = "DEL".parse::<MedialParam>() else {
            unreachable!();
        };
        builder.push_medial(subcmd);
        let Ok(finalp) = FinalParam::try_from(join_with_space(&self.capabilities).to_string())
        else {
            unreachable!()
        };
        let params = builder.with_final(finalp);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        format!(
            "CAP {} DEL :{}",
            self.target,
            join_with_space(&self.capabilities)
        )
    }
}

impl From<CapDel> for Cap {
    fn from(value: CapDel) -> Cap {
        Cap::Del(value)
    }
}

impl From<CapDel> for ClientMessage {
    fn from(value: CapDel) -> ClientMessage {
        ClientMessage::from(Cap::from(value))
    }
}

impl From<CapDel> for Message {
    fn from(value: CapDel) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<CapDel> for RawMessage {
    fn from(value: CapDel) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct Capability(String);

validstr!(Capability, ParseCapabilityError, validate_capability);

fn validate_capability(s: &str) -> Result<(), ParseCapabilityError> {
    if s.is_empty() {
        Err(ParseCapabilityError::Empty)
    } else if s.starts_with('-') {
        Err(ParseCapabilityError::BadStart)
    } else if s.contains(['\0', '\r', '\n', ' ', '=']) {
        Err(ParseCapabilityError::BadCharacter)
    } else {
        Ok(())
    }
}

impl From<Capability> for MedialParam {
    fn from(value: Capability) -> MedialParam {
        MedialParam::try_from(value.into_inner()).expect("Capability should be valid MedialParam")
    }
}

impl From<Capability> for FinalParam {
    fn from(value: Capability) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub enum ParseCapabilityError {
    #[error("capabilities cannot be empty")]
    Empty,
    #[error("capabilities cannot start with '-'")]
    BadStart,
    #[error("capabilities cannot contain NUL, CR, LF, SPACE, or =")]
    BadCharacter,
}

#[derive(Clone, Eq, Hash, PartialEq)]
pub struct CapabilityValue(String);

validstr!(
    CapabilityValue,
    ParseCapabilityValueError,
    validate_capability_value
);

fn validate_capability_value(s: &str) -> Result<(), ParseCapabilityValueError> {
    if s.contains(['\0', '\r', '\n', ' ']) {
        Err(ParseCapabilityValueError)
    } else {
        Ok(())
    }
}

impl From<CapabilityValue> for MedialParam {
    fn from(value: CapabilityValue) -> MedialParam {
        MedialParam::try_from(value.into_inner())
            .expect("Capability value should be valid MedialParam")
    }
}

impl From<CapabilityValue> for FinalParam {
    fn from(value: CapabilityValue) -> FinalParam {
        FinalParam::from(MedialParam::from(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("capability values cannot contain NUL, CR, LF, or SPACE")]
pub struct ParseCapabilityValueError;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CapabilityRequest {
    pub capability: Capability,
    pub disable: bool,
}

impl CapabilityRequest {
    pub fn enable(capability: Capability) -> CapabilityRequest {
        CapabilityRequest {
            capability,
            disable: false,
        }
    }

    pub fn disable(capability: Capability) -> CapabilityRequest {
        CapabilityRequest {
            capability,
            disable: true,
        }
    }
}

impl fmt::Display for CapabilityRequest {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.disable {
            write!(f, "-")?;
        }
        write!(f, "{}", self.capability)
    }
}

impl std::str::FromStr for CapabilityRequest {
    type Err = ParseCapabilityError;

    fn from_str(s: &str) -> Result<CapabilityRequest, ParseCapabilityError> {
        let (s, disable) = if let Some(s) = s.strip_prefix('-') {
            (s, true)
        } else {
            (s, false)
        };
        let capability = s.parse::<Capability>()?;
        Ok(CapabilityRequest {
            capability,
            disable,
        })
    }
}

impl TryFrom<String> for CapabilityRequest {
    type Error = TryFromStringError<ParseCapabilityError>;

    fn try_from(
        string: String,
    ) -> Result<CapabilityRequest, TryFromStringError<ParseCapabilityError>> {
        match string.parse() {
            Ok(src) => Ok(src),
            Err(inner) => Err(TryFromStringError { inner, string }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Payload;
    use assert_matches::assert_matches;

    #[test]
    fn ls_request_version() {
        let msg = "CAP LS 302".parse::<Message>().unwrap();
        assert_eq!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::LsRequest(CapLsRequest {
                version: Some(302)
            })))
        );
    }

    #[test]
    fn ls_response_continued() {
        let msg = "CAP * LS * :cap-notify server-time example.org/dummy-cap=dummyvalue example.org/second-dummy-cap".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::LsResponse(cap))) => {
                assert!(cap.target.is_star());
                assert!(cap.continued);
                assert_eq!(cap.capabilities, [
                    ("cap-notify".parse::<Capability>().unwrap(), None),
                    ("server-time".parse::<Capability>().unwrap(), None),
                    ("example.org/dummy-cap".parse::<Capability>().unwrap(), Some("dummyvalue".parse::<CapabilityValue>().unwrap())),
                    ("example.org/second-dummy-cap".parse::<Capability>().unwrap(), None),
                ]);
            }
        );
    }

    #[test]
    fn ls_response() {
        let msg = "CAP * LS :userhost-in-names sasl=EXTERNAL,DH-AES,DH-BLOWFISH,ECDSA-NIST256P-CHALLENGE,PLAIN".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::LsResponse(cap))) => {
                assert!(cap.target.is_star());
                assert!(!cap.continued);
                assert_eq!(cap.capabilities, [
                    ("userhost-in-names".parse::<Capability>().unwrap(), None),
                    ("sasl".parse::<Capability>().unwrap(), Some("EXTERNAL,DH-AES,DH-BLOWFISH,ECDSA-NIST256P-CHALLENGE,PLAIN".parse::<CapabilityValue>().unwrap())),
                ]);
            }
        );
    }

    #[test]
    fn list_request() {
        let msg = "CAP LIST".parse::<Message>().unwrap();
        assert_eq!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::ListRequest(CapListRequest)))
        );
    }

    #[test]
    fn list_response_continued() {
        let msg = "CAP modernclient LIST * :example.org/example-cap example.org/second-example-cap account-notify".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::ListResponse(cap))) => {
                assert_eq!(cap.target, "modernclient");
                assert!(cap.continued);
                assert_eq!(cap.capabilities, [
                    "example.org/example-cap".parse::<Capability>().unwrap(),
                    "example.org/second-example-cap".parse::<Capability>().unwrap(),
                    "account-notify".parse::<Capability>().unwrap(),
                ]);
            }
        );
    }

    #[test]
    fn list_response() {
        let msg = "CAP modernclient LIST :invite-notify batch example.org/third-example-cap"
            .parse::<Message>()
            .unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::ListResponse(cap))) => {
                assert_eq!(cap.target, "modernclient");
                assert!(!cap.continued);
                assert_eq!(cap.capabilities, [
                    "invite-notify".parse::<Capability>().unwrap(),
                    "batch".parse::<Capability>().unwrap(),
                    "example.org/third-example-cap".parse::<Capability>().unwrap(),
                ]);
            }
        );
    }

    #[test]
    fn req() {
        let msg = "CAP REQ :multi-prefix sasl".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::Req(cap))) => {
                assert_eq!(cap.capabilities, [
                    CapabilityRequest {capability: "multi-prefix".parse::<Capability>().unwrap(), disable: false},
                    CapabilityRequest {capability: "sasl".parse::<Capability>().unwrap(), disable: false},
                ]);
            }
        );
    }

    #[test]
    fn ack() {
        let msg = "CAP * ACK :multi-prefix sasl".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::Ack(cap))) => {
                assert!(cap.target.is_star());
                assert_eq!(cap.capabilities, [
                    CapabilityRequest {capability: "multi-prefix".parse::<Capability>().unwrap(), disable: false},
                    CapabilityRequest {capability: "sasl".parse::<Capability>().unwrap(), disable: false},
                ]);
            }
        );
    }

    #[test]
    fn req_disable() {
        let msg = "CAP REQ :-userhost-in-names".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::Req(cap))) => {
                assert_eq!(cap.capabilities, [
                    CapabilityRequest {capability: "userhost-in-names".parse::<Capability>().unwrap(), disable: true},
                ]);
            }
        );
    }

    #[test]
    fn ack_disable() {
        let msg = "CAP * ACK :-userhost-in-names".parse::<Message>().unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::Ack(cap))) => {
                assert!(cap.target.is_star());
                assert_eq!(cap.capabilities, [
                    CapabilityRequest {capability: "userhost-in-names".parse::<Capability>().unwrap(), disable: true},
                ]);
            }
        );
    }

    #[test]
    fn nak() {
        let msg = "CAP * NAK :multi-prefix sasl ex3"
            .parse::<Message>()
            .unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::Nak(cap))) => {
                assert!(cap.target.is_star());
                assert_eq!(cap.capabilities, [
                    "multi-prefix".parse::<Capability>().unwrap(),
                    "sasl".parse::<Capability>().unwrap(),
                    "ex3".parse::<Capability>().unwrap(),
                ]);
            }
        );
    }

    #[test]
    fn end() {
        let msg = "CAP END".parse::<Message>().unwrap();
        assert_eq!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::End(CapEnd)))
        );
    }

    #[test]
    fn new() {
        let msg = ":irc.example.com CAP modernclient NEW :batch"
            .parse::<Message>()
            .unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::New(cap))) => {
                assert_eq!(cap.target, "modernclient");
                assert_eq!(cap.capabilities, [
                    "batch".parse::<Capability>().unwrap(),
                ]);
            }
        );
    }

    #[test]
    fn del() {
        let msg =
            ":irc.example.com CAP modernclient DEL :userhost-in-names multi-prefix away-notify"
                .parse::<Message>()
                .unwrap();
        assert_matches!(
            msg.payload,
            Payload::ClientMessage(ClientMessage::Cap(Cap::Del(cap))) => {
                assert_eq!(cap.target, "modernclient");
                assert_eq!(cap.capabilities, [
                    "userhost-in-names".parse::<Capability>().unwrap(),
                    "multi-prefix".parse::<Capability>().unwrap(),
                    "away-notify".parse::<Capability>().unwrap(),
                ]);
            }
        );
    }
}
