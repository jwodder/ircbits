#![expect(dead_code, unused_variables, clippy::todo)]
use super::{SaslError, SaslFlow};
use bytes::Bytes;
use irctext::clientmsgs::Authenticate;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashAlgo {
    Sha1,
    Sha512,
}

impl HashAlgo {
    fn hash(self, bs: &[u8]) -> Bytes {
        todo!()
    }

    fn hmac(self, key: &[u8], s: &[u8]) -> Bytes {
        todo!()
    }

    // RFC 5802's "Hi()"
    fn iter_hash(self, s: &[u8], salt: &[u8], i: u32) -> Bytes {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScramSasl {
    hash: HashAlgo,
    state: State,
}

impl ScramSasl {
    pub fn new(nickname: &str, password: &str, hash: HashAlgo) -> ScramSasl {
        todo!()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Start,
    AwaitingPlus,
    GotPlus, // about to send first client-first-message
    AwitingServerFirstMsg,
    GotServerFirstMsg, // about to send client-final-message
    AwaitingServerFinalMsg,
    Done,
    Void,
}

impl SaslFlow for ScramSasl {
    fn handle_message(&mut self, msg: Authenticate) -> Result<(), SaslError> {
        todo!()
    }

    fn get_output(&mut self) -> Vec<Authenticate> {
        todo!()
    }

    fn is_done(&self) -> bool {
        todo!()
    }
}
