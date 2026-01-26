#![expect(dead_code, unused_variables, unreachable_code, clippy::todo)]
use super::{SaslError, SaslFlow, SaslMechanism};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use irctext::{
    TrailingParam,
    clientmsgs::{Authenticate, ClientMessageParts},
    types::Nickname,
};
use rand::{
    SeedableRng,
    distr::{Alphanumeric, Distribution},
    rngs::StdRng,
};
use replace_with::replace_with_and_return;
use thiserror::Error;

const CLIENT_NONCE_LENGTH: usize = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HashAlgo {
    Sha1,
    Sha512,
}

impl HashAlgo {
    fn mechanism(self) -> SaslMechanism {
        match self {
            HashAlgo::Sha1 => SaslMechanism::ScramSha1,
            HashAlgo::Sha512 => SaslMechanism::ScramSha512,
        }
    }

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
    pub fn new(
        nickname: &Nickname,
        password: &str,
        hash: HashAlgo,
    ) -> Result<ScramSasl, SaslError> {
        let Ok(mech) = hash.mechanism().as_ref().parse::<TrailingParam>() else {
            unreachable!("SaslMechanism strings should be valid trailing params");
        };
        let mech_msg = Authenticate::new(mech);
        let nonce = generate_nonce();
        let username = nickname.as_str().parse::<Username>()?;
        let clifirst = ClientFirstMessage {
            authzid: nickname.clone(),
            username,
            nonce: nonce.clone(),
        };
        Ok(ScramSasl {
            hash,
            state: State::Start {
                mech_msg,
                nonce,
                clifirst,
                password: password.to_owned(),
            },
        })
    }
}

impl SaslFlow for ScramSasl {
    fn handle_message(&mut self, msg: Authenticate) -> Result<(), SaslError> {
        replace_with_and_return(
            &mut self.state,
            || State::Void,
            |state| match state {
                State::Start { .. } => {
                    panic!("handle_message() called before calling get_output()")
                }
                State::AwaitingPlus {
                    nonce,
                    clifirst,
                    password,
                } => {
                    if msg.parameter() == "+" {
                        (
                            Ok(()),
                            State::GotPlus {
                                nonce,
                                clifirst,
                                password,
                            },
                        )
                    } else {
                        (
                            Err(SaslError::Unexpected {
                                expecting: r#""AUTHENTICATE +""#,
                                msg: msg.to_irc_line(),
                            }),
                            State::Done,
                        )
                    }
                }
                State::GotPlus { .. } => {
                    panic!("handle_message() called before calling get_output()")
                }
                State::AwaitingServerFirstMsg {
                    nonce,
                    password,
                    mut input,
                } => {
                    let payload = msg.parameter().as_str();
                    if payload != "+" {
                        input.push_str(payload);
                    }
                    if payload.len() < 400 {
                        let bs = match STANDARD.decode(input) {
                            Ok(bs) => bs,
                            Err(e) => return (Err(SaslError::Base64Decode(e)), State::Done),
                        };
                        let s = match String::from_utf8(bs) {
                            Ok(s) => s,
                            Err(e) => todo!("Return some error"),
                        };
                        // Parse to ServerFirstMessage
                        // Do scram computation
                        // Assemble ClientFinalMessage
                        let clifinal = todo!();
                        let server_signature = todo!();
                        (
                            Ok(()),
                            State::GotServerFirstMsg {
                                clifinal,
                                server_signature,
                            },
                        )
                    } else {
                        (
                            Ok(()),
                            State::AwaitingServerFirstMsg {
                                nonce,
                                password,
                                input,
                            },
                        )
                    }
                }
                State::GotServerFirstMsg { .. } => {
                    panic!("handle_message() called before calling get_output()")
                }
                State::AwaitingServerFinalMsg { .. } => todo!(),
                State::Done => panic!("get_output() called on Done SASL state"),
                State::Void => panic!("get_output() called on Void SASL state"),
            },
        )
    }

    fn get_output(&mut self) -> Vec<Authenticate> {
        replace_with_and_return(
            &mut self.state,
            || State::Void,
            |state| match state {
                State::Start {
                    mech_msg,
                    nonce,
                    clifirst,
                    password,
                } => (
                    vec![mech_msg],
                    State::AwaitingPlus {
                        nonce,
                        clifirst,
                        password,
                    },
                ),
                State::AwaitingPlus { .. } => (Vec::new(), state),
                State::GotPlus {
                    nonce,
                    clifirst,
                    password,
                } => (
                    clifirst.into_auth_msgs(),
                    State::AwaitingServerFirstMsg {
                        nonce,
                        password,
                        input: String::new(),
                    },
                ),
                State::AwaitingServerFirstMsg { .. } => (Vec::new(), state),
                State::GotServerFirstMsg {
                    clifinal,
                    server_signature,
                } => (
                    clifinal.into_auth_msgs(),
                    State::AwaitingServerFinalMsg {
                        server_signature,
                        input: String::new(),
                    },
                ),
                State::AwaitingServerFinalMsg { .. } => (Vec::new(), state),
                State::Done => (Vec::new(), State::Done),
                State::Void => panic!("get_output() called on Void SASL state"),
            },
        )
    }

    fn is_done(&self) -> bool {
        self.state == State::Done
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Start {
        mech_msg: Authenticate,
        nonce: String,
        clifirst: ClientFirstMessage,
        password: String,
    },
    AwaitingPlus {
        nonce: String,
        clifirst: ClientFirstMessage,
        password: String,
    },
    GotPlus {
        // about to send first client-first-message
        nonce: String,
        clifirst: ClientFirstMessage,
        password: String,
    },
    AwaitingServerFirstMsg {
        nonce: String,
        password: String,
        /// Undecoded base 64 formed by concatenating the payloads of the
        /// Authenticate messages received so far
        input: String,
    },
    GotServerFirstMsg {
        // about to send client-final-message
        clifinal: ClientFinalMessage,
        server_signature: Bytes,
    },
    AwaitingServerFinalMsg {
        server_signature: Bytes,
        /// Undecoded base 64 formed by concatenating the payloads of the
        /// Authenticate messages received so far
        input: String,
    },
    Done,
    Void,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClientFirstMessage {
    // > The "gs2-authzid" holds the SASL authorization identity.  It is encoded
    // > using UTF-8 [RFC3629] with three exceptions:
    // >
    // > - The NUL character is forbidden as required by section 3.4.1 of [RFC4422].
    // >
    // > - The server MUST replace any "," (comma) in the string with "=2C".
    // >
    // > - The server MUST replace any "=" (equals) in the string with "=3D".
    //
    // — RFC 5801, §4
    authzid: Nickname,
    username: Username,
    nonce: String,
}

impl ClientFirstMessage {
    fn into_auth_msgs(self) -> Vec<Authenticate> {
        todo!()
    }
}

// TODO: impl fmt::Display for ClientFirstMessage

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServerFirstMessage {
    nonce: String,
    salt: Bytes,
    iteration_count: u32,
}

// TODO: impl std::str::FromStr for ServerFirstMessage

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClientFinalMessage {
    nonce: String,
    proof: Bytes,
}

impl ClientFinalMessage {
    fn into_auth_msgs(self) -> Vec<Authenticate> {
        todo!()
    }
}

// TODO: impl fmt::Display for ClientFinalMessage

#[derive(Clone, Debug, Eq, PartialEq)]
enum ServerFinalMessage {
    Success { verifier: Bytes },
    Error { message: String },
}

// TODO: impl std::str::FromStr for ServerFinalMessage

#[derive(Clone, Debug, Eq, PartialEq)]
struct Username(String);

impl Username {
    fn escaped(&self) -> std::borrow::Cow<'_, str> {
        // > The characters ',' or '=' in usernames are sent as '=2C' and '=3D'
        // > respectively.  If the server receives a username that contains '='
        // > not followed by either '2C' or '3D', then the server MUST fail the
        // > authentication.
        //
        // — RFC 5802, §5.1
        todo!()
    }
}

// > Before sending the username to the server, the client SHOULD prepare the
// > username using the "SASLprep" profile [RFC4013] of the "stringprep"
// > algorithm [RFC3454] treating it as a query string (i.e., unassigned Unicode
// > code points are allowed).  If the preparation of the username fails or
// > results in an empty string, the client SHOULD abort the authentication
// > exchange.
//
// — RFC 5802, §5.1
impl std::str::FromStr for Username {
    type Err = PrepareUsernameError;

    fn from_str(s: &str) -> Result<Username, PrepareUsernameError> {
        let s = stringprep::saslprep(s)?;
        if s.is_empty() {
            Err(PrepareUsernameError::Empty)
        } else {
            Ok(Username(s.into_owned()))
        }
    }
}

#[derive(Debug, Error)]
pub enum PrepareUsernameError {
    #[error("stringprep algorithm failed")]
    Stringprep(#[from] stringprep::Error),
    #[error("stringprep resulted in an empty string")]
    Empty,
}

fn generate_nonce() -> String {
    let mut rng = StdRng::from_os_rng();
    Alphanumeric
        .sample_iter(&mut rng)
        .take(CLIENT_NONCE_LENGTH)
        .map(char::from)
        .collect()
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Computation {
    client_proof: Bytes,
    server_signature: Bytes,
}

fn compute_scram(
    hash: HashAlgo,
    password: &str,
    salt: &[u8],
    iteration_count: u32,
    auth_message: &str,
) -> Result<Computation, SaslError> {
    let normed_password = stringprep::saslprep(password).map_err(SaslError::PreparePassword)?;
    let salted_password = hash.iter_hash(normed_password.as_bytes(), salt, iteration_count);
    let client_key = hash.hmac(&salted_password, b"Client Key");
    let stored_key = hash.hash(&client_key);
    let client_signature = hash.hmac(&stored_key, auth_message.as_bytes());
    let client_proof = std::iter::zip(client_key, client_signature)
        .map(|(a, b)| a ^ b)
        .collect::<Bytes>();
    let server_key = hash.hmac(&salted_password, b"Server Key");
    let server_signature = hash.hmac(&server_key, auth_message.as_bytes());
    Ok(Computation {
        client_proof,
        server_signature,
    })
}
