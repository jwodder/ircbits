use super::{ClientMessage, ClientMessageError, ClientMessageParts};
use crate::types::{ModeString, ModeTarget};
use crate::{Message, ParameterList, ParameterListSizeError, RawMessage, Verb};
use std::fmt::Write;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Mode {
    target: ModeTarget,
    modestring: Option<ModeString>,
    arguments: ParameterList,
}

impl Mode {
    pub fn new(target: ModeTarget) -> Mode {
        Mode {
            target,
            modestring: None,
            arguments: ParameterList::default(),
        }
    }

    pub fn new_with_modestring(target: ModeTarget, modestring: ModeString) -> Mode {
        Mode {
            target,
            modestring: Some(modestring),
            arguments: ParameterList::default(),
        }
    }

    pub fn new_with_arguments(
        target: ModeTarget,
        modestring: ModeString,
        arguments: ParameterList,
    ) -> Mode {
        Mode {
            target,
            modestring: Some(modestring),
            arguments,
        }
    }

    pub fn target(&self) -> &ModeTarget {
        &self.target
    }

    pub fn modestring(&self) -> Option<&ModeString> {
        self.modestring.as_ref()
    }

    pub fn arguments(&self) -> &ParameterList {
        &self.arguments
    }
}

impl ClientMessageParts for Mode {
    fn into_parts(self) -> (Verb, ParameterList) {
        let builder = ParameterList::builder().with_medial(self.target);
        let params = if let Some(modestring) = self.modestring {
            builder.with_medial(modestring).with_list(self.arguments)
        } else {
            builder.finish()
        };
        (Verb::Mode, params)
    }

    fn to_irc_line(&self) -> String {
        let mut s = format!("MODE {}", self.target);
        if let Some(ref modestring) = self.modestring {
            write!(&mut s, " {modestring}").unwrap();
            if !self.arguments.is_empty() {
                write!(&mut s, " {}", self.arguments).unwrap();
            }
        }
        s
    }
}

impl From<Mode> for Message {
    fn from(value: Mode) -> Message {
        Message::from(ClientMessage::from(value))
    }
}

impl From<Mode> for RawMessage {
    fn from(value: Mode) -> RawMessage {
        RawMessage::from(ClientMessage::from(value))
    }
}

impl TryFrom<ParameterList> for Mode {
    type Error = ClientMessageError;

    fn try_from(params: ParameterList) -> Result<Mode, ClientMessageError> {
        let mut iter = params.into_iter();
        let p1 = iter.next().ok_or(ParameterListSizeError::Exact {
            requested: 1,
            received: 0,
        })?;
        let target = ModeTarget::try_from(p1.into_inner())?;
        let modestring = if let Some(p2) = iter.next() {
            Some(ModeString::try_from(p2.into_inner())?)
        } else {
            None
        };
        let arguments = iter.into_parameter_list();
        Ok(Mode {
            target,
            modestring,
            arguments,
        })
    }
}
