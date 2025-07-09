use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::ReplyTarget;
use crate::{
    FinalParam, MedialParam, Message, ParameterList, ParameterListSizeError, RawMessage, Verb,
};
use std::fmt::Write;

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Cap {
    target: Option<ReplyTarget>,
    subcommand: MedialParam,
    // Whether there's an asterisk parameter between the subcommand and the
    // actual parameter:
    continued: bool,
    parameter: Option<FinalParam>,
}

impl Cap {
    pub fn new(subcommand: MedialParam) -> Cap {
        Cap {
            target: None,
            subcommand,
            continued: false,
            parameter: None,
        }
    }

    pub fn with_parameter(mut self, parameter: FinalParam) -> Cap {
        self.parameter = Some(parameter);
        self
    }

    pub fn with_target<P: Into<ReplyTarget>>(mut self, target: P) -> Cap {
        self.target = Some(target.into());
        self
    }

    pub fn with_continued(mut self, yesno: bool) -> Cap {
        self.continued = yesno;
        self
    }

    pub fn subcommand(&self) -> &MedialParam {
        &self.subcommand
    }

    pub fn target(&self) -> Option<&ReplyTarget> {
        self.target.as_ref()
    }

    pub fn parameter(&self) -> Option<&FinalParam> {
        self.parameter.as_ref()
    }

    pub fn continued(&self) -> bool {
        self.continued
    }
}

impl ClientMessageParts for Cap {
    fn into_parts(self) -> (Verb, ParameterList) {
        let mut builder = ParameterList::builder();
        if let Some(target) = self.target {
            builder.push_medial(
                MedialParam::try_from(target.into_inner())
                    .expect("CapTarget should be valid MedialParam"),
            );
        }
        builder.push_medial(self.subcommand);
        if self.continued {
            builder.push_medial(
                "*".parse::<MedialParam>()
                    .expect(r#""*" should be a valid MedialParam"#),
            );
        }
        let params = builder.maybe_with_final(self.parameter);
        (Verb::Cap, params)
    }

    fn to_irc_line(&self) -> String {
        let mut s = String::from("CAP");
        if let Some(ref target) = self.target {
            write!(&mut s, " {target}").unwrap();
        }
        write!(&mut s, " {}", self.subcommand).unwrap();
        if self.continued {
            write!(&mut s, " *").unwrap();
        }
        if let Some(ref param) = self.parameter {
            write!(&mut s, " :{param}").unwrap();
        }
        s
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
                let Ok((p1,)): Result<(FinalParam,), _> = params.try_into() else {
                    unreachable!("ParameterList should be convertible to 1-tuple when len is 1");
                };
                let subcommand = MedialParam::try_from(p1.into_inner())?;
                Ok(Cap::new(subcommand))
            }
            2 => {
                // CAP <subcommand> <parameter>
                let Ok((subcommand, parameter)): Result<(MedialParam, FinalParam), _> =
                    params.try_into()
                else {
                    unreachable!("ParameterList should be convertible to 2-tuple when len is 2");
                };
                Ok(Cap::new(subcommand).with_parameter(parameter))
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
                Ok(Cap::new(subcommand)
                    .with_parameter(parameter)
                    .with_target(target))
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
                Ok(Cap::new(subcommand)
                    .with_parameter(parameter)
                    .with_target(target)
                    .with_continued(true))
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
