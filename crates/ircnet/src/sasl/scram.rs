use super::{SaslError, SaslFlow, SaslMechanism};
use base64::{Engine, engine::general_purpose::STANDARD};
use bytes::Bytes;
use enum_dispatch::enum_dispatch;
use hmac::{Hmac, Mac};
use irctext::{
    TrailingParam,
    clientmsgs::{Authenticate, ClientMessageParts},
    types::Nickname,
};
use pbkdf2::pbkdf2_hmac_array;
use rand::{
    SeedableRng,
    distr::{Alphanumeric, Distribution},
    rngs::StdRng,
};
use replace_with::replace_with_and_return;
use sha1::{Digest as _, Sha1};
use sha2::Sha512;
use std::fmt;
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
        match self {
            HashAlgo::Sha1 => Bytes::from_iter(Sha1::digest(bs)),
            HashAlgo::Sha512 => Bytes::from_iter(Sha512::digest(bs)),
        }
    }

    fn hmac(self, key: &[u8], s: &[u8]) -> Bytes {
        match self {
            HashAlgo::Sha1 => {
                let mut mac =
                    Hmac::<Sha1>::new_from_slice(key).expect("any key length should be accepted");
                mac.update(s);
                Bytes::from_iter(mac.finalize().into_bytes())
            }
            HashAlgo::Sha512 => {
                let mut mac =
                    Hmac::<Sha512>::new_from_slice(key).expect("any key length should be accepted");
                mac.update(s);
                Bytes::from_iter(mac.finalize().into_bytes())
            }
        }
    }

    // RFC 5802's "Hi()"
    fn iter_hash(self, s: &[u8], salt: &[u8], i: u32) -> Bytes {
        match self {
            HashAlgo::Sha1 => Bytes::from_iter(pbkdf2_hmac_array::<Sha1, 20>(s, salt, i)),
            HashAlgo::Sha512 => Bytes::from_iter(pbkdf2_hmac_array::<Sha512, 64>(s, salt, i)),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScramSasl {
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
        let password = password.parse::<Password>()?;
        Ok(ScramSasl {
            state: State::Start(Start {
                hash,
                mech_msg,
                nonce,
                authzid: nickname.clone(),
                username,
                password,
            }),
        })
    }
}

impl SaslFlow for ScramSasl {
    fn handle_message(&mut self, msg: Authenticate) -> Result<(), SaslError> {
        replace_with_and_return(
            &mut self.state,
            || State::Error(Error),
            move |state| match state.handle_message(msg) {
                Ok(state) => (Ok(()), state),
                Err(e) => (Err(e), State::Error(Error)),
            },
        )
    }

    fn get_output(&mut self) -> Vec<Authenticate> {
        replace_with_and_return(&mut self.state, || State::Error(Error), State::get_output)
    }

    fn is_done(&self) -> bool {
        self.state.is_done()
    }
}

#[enum_dispatch]
trait ScramState {
    fn handle_message(self, msg: Authenticate) -> Result<State, SaslError>;
    fn get_output(self) -> (Vec<Authenticate>, State);
    fn is_done(&self) -> bool;
}

#[enum_dispatch(ScramState)]
#[derive(Clone, Debug, Eq, PartialEq)]
enum State {
    Start,
    AwaitingPlus,
    GotPlus,
    AwaitingServerFirstMsg,
    GotServerFirstMsg,
    AwaitingServerFinalMsg,
    Finishing,
    Done,
    Error,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Start {
    hash: HashAlgo,
    mech_msg: Authenticate,
    nonce: String,
    authzid: AuthzId,
    username: Username,
    password: Password,
}

impl ScramState for Start {
    fn handle_message(self, _msg: Authenticate) -> Result<State, SaslError> {
        panic!("handle_message() called before calling get_output()")
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        (
            vec![self.mech_msg],
            AwaitingPlus {
                hash: self.hash,
                nonce: self.nonce,
                authzid: self.authzid,
                username: self.username,
                password: self.password,
            }
            .into(),
        )
    }

    fn is_done(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AwaitingPlus {
    hash: HashAlgo,
    nonce: String,
    authzid: AuthzId,
    username: Username,
    password: Password,
}

impl ScramState for AwaitingPlus {
    fn handle_message(self, msg: Authenticate) -> Result<State, SaslError> {
        if msg.parameter() == "+" {
            Ok(GotPlus {
                hash: self.hash,
                nonce: self.nonce,
                authzid: self.authzid,
                username: self.username,
                password: self.password,
            }
            .into())
        } else {
            Err(SaslError::Unexpected {
                expecting: r#""AUTHENTICATE +""#,
                msg: msg.to_irc_line(),
            })
        }
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        (Vec::new(), self.into())
    }

    fn is_done(&self) -> bool {
        false
    }
}

// About to send first client-first-message
#[derive(Clone, Debug, Eq, PartialEq)]
struct GotPlus {
    hash: HashAlgo,
    nonce: String,
    authzid: AuthzId,
    username: Username,
    password: Password,
}

impl ScramState for GotPlus {
    fn handle_message(self, _msg: Authenticate) -> Result<State, SaslError> {
        panic!("handle_message() called before calling get_output()")
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        let msgs = ClientFirstMessage {
            authzid: &self.authzid,
            username: &self.username,
            nonce: &self.nonce,
        }
        .to_auth_msgs();
        (
            msgs,
            AwaitingServerFirstMsg {
                hash: self.hash,
                nonce: self.nonce,
                authzid: self.authzid,
                username: self.username,
                password: self.password,
                input: String::new(),
            }
            .into(),
        )
    }

    fn is_done(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AwaitingServerFirstMsg {
    hash: HashAlgo,
    nonce: String,
    authzid: AuthzId,
    username: Username,
    password: Password,
    /// Undecoded base 64 formed by concatenating the payloads of the
    /// Authenticate messages received so far
    input: String,
}

impl ScramState for AwaitingServerFirstMsg {
    fn handle_message(mut self, msg: Authenticate) -> Result<State, SaslError> {
        let payload = msg.parameter().as_str();
        if payload != "+" {
            self.input.push_str(payload);
        }
        if payload.len() < 400 {
            let bs = STANDARD.decode(&self.input)?;
            let s = std::str::from_utf8(&bs)?;
            let server_first = s.parse::<ServerFirstMessage>()?;

            // AuthMessage     := client-first-message-bare + "," +
            //                        server-first-message + "," +
            //                        client-final-message-without-proof
            //
            // client-first-message-bare = username "," nonce
            //
            // server-first-message =
            //       [reserved-mext ","] nonce "," salt ","
            //       iteration-count ["," extensions]
            //
            // client-final-message-without-proof =
            //       channel-binding "," nonce
            //
            // channel-binding = "c=" base64
            //       ;; base64 encoding of cbind-input.
            //
            // cbind-input   = gs2-header [ cbind-data ]
            //       ;; cbind-data MUST be present for
            //       ;; gs2-cbind-flag of "p" and MUST be absent
            //       ;; for "y" or "n".

            let client_nonce = self.nonce;
            let final_nonce = server_first.nonce;
            if !final_nonce.starts_with(&client_nonce) {
                return Err(SaslError::Nonce);
            }
            let cbind_input = format!("n,a={},", Gs2Escaped(&self.authzid));
            let auth_message = format!(
                "n={username},r={client_nonce},{s},c={binding},r={final_nonce}",
                username = Gs2Escaped(self.username.as_str()),
                binding = STANDARD.encode(&cbind_input),
            );
            let Computation {
                client_proof,
                server_signature,
            } = compute_scram(
                self.hash,
                &self.password,
                &server_first.salt,
                server_first.iteration_count,
                &auth_message,
            );
            Ok(GotServerFirstMsg {
                authzid: self.authzid,
                nonce: final_nonce,
                client_proof,
                server_signature,
            }
            .into())
        } else {
            Ok(self.into())
        }
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        (Vec::new(), self.into())
    }

    fn is_done(&self) -> bool {
        false
    }
}

// About to send client-final-message
#[derive(Clone, Debug, Eq, PartialEq)]
struct GotServerFirstMsg {
    authzid: AuthzId,
    nonce: String,
    client_proof: Bytes,
    server_signature: Bytes,
}

impl ScramState for GotServerFirstMsg {
    fn handle_message(self, _msg: Authenticate) -> Result<State, SaslError> {
        panic!("handle_message() called before calling get_output()")
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        let msgs = ClientFinalMessage {
            authzid: &self.authzid,
            nonce: &self.nonce,
            proof: &self.client_proof,
        }
        .to_auth_msgs();
        (
            msgs,
            AwaitingServerFinalMsg {
                server_signature: self.server_signature,
                input: String::new(),
            }
            .into(),
        )
    }

    fn is_done(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct AwaitingServerFinalMsg {
    server_signature: Bytes,
    /// Undecoded base 64 formed by concatenating the payloads of the
    /// Authenticate messages received so far
    input: String,
}

impl ScramState for AwaitingServerFinalMsg {
    fn handle_message(mut self, msg: Authenticate) -> Result<State, SaslError> {
        let payload = msg.parameter().as_str();
        if payload != "+" {
            self.input.push_str(payload);
        }
        if payload.len() < 400 {
            let bs = STANDARD.decode(&self.input)?;
            let s = std::str::from_utf8(&bs)?;
            match s.parse::<ServerFinalMessage>()? {
                ServerFinalMessage::Success { verifier } => {
                    if verifier == self.server_signature {
                        Ok(Finishing.into())
                    } else {
                        Err(SaslError::Signature)
                    }
                }
                ServerFinalMessage::Error { message } => Err(SaslError::Server(message)),
            }
        } else {
            Ok(self.into())
        }
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        (Vec::new(), self.into())
    }

    fn is_done(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Finishing;

impl ScramState for Finishing {
    fn handle_message(self, _msg: Authenticate) -> Result<State, SaslError> {
        panic!("handle_message() called before calling get_output()")
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        (vec![Authenticate::new_empty()], Done.into())
    }

    fn is_done(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Done;

impl ScramState for Done {
    fn handle_message(self, _msg: Authenticate) -> Result<State, SaslError> {
        panic!("handle_message() called on Done state")
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        (Vec::new(), self.into())
    }

    fn is_done(&self) -> bool {
        true
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Error;

impl ScramState for Error {
    fn handle_message(self, _msg: Authenticate) -> Result<State, SaslError> {
        panic!("handle_message() called on Error state")
    }

    fn get_output(self) -> (Vec<Authenticate>, State) {
        panic!("get_output() called on Error state")
    }

    fn is_done(&self) -> bool {
        panic!("is_done() called on Error state")
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClientFirstMessage<'a> {
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
    authzid: &'a AuthzId,
    username: &'a Username,
    nonce: &'a str,
}

impl ClientFirstMessage<'_> {
    fn to_auth_msgs(&self) -> Vec<Authenticate> {
        Authenticate::new_encoded(self.to_string().as_bytes())
    }
}

impl fmt::Display for ClientFirstMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // gs2-header
        write!(f, "n,a={},", Gs2Escaped(self.authzid))?;
        // client-first-message-bare
        write!(
            f,
            "n={},r={}",
            Gs2Escaped(self.username.as_str()),
            self.nonce
        )?;
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ServerFirstMessage {
    nonce: String,
    salt: Bytes,
    iteration_count: u32,
}

impl std::str::FromStr for ServerFirstMessage {
    type Err = SaslError;

    fn from_str(s: &str) -> Result<ServerFirstMessage, Self::Err> {
        let mut ss = s;
        let nonce = if let Some(("r", r)) = parse_gs2_pair(&mut ss)? {
            r.to_owned()
        } else {
            return Err(SaslError::Parse);
        };
        let salt = if let Some(("s", b64salt)) = parse_gs2_pair(&mut ss)? {
            Bytes::from(STANDARD.decode(b64salt)?)
        } else {
            return Err(SaslError::Parse);
        };
        let iteration_count = if let Some(("i", i)) = parse_gs2_pair(&mut ss)? {
            i.parse::<u32>().map_err(|_| SaslError::Parse)?
        } else {
            return Err(SaslError::Parse);
        };
        Ok(ServerFirstMessage {
            nonce,
            salt,
            iteration_count,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ClientFinalMessage<'a> {
    authzid: &'a AuthzId,
    nonce: &'a str,
    proof: &'a [u8],
}

impl ClientFinalMessage<'_> {
    fn to_auth_msgs(&self) -> Vec<Authenticate> {
        Authenticate::new_encoded(self.to_string().as_bytes())
    }
}

impl fmt::Display for ClientFinalMessage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let cbind_input = format!("n,a={},", Gs2Escaped(self.authzid));
        write!(
            f,
            "c={},r={},p={}",
            STANDARD.encode(&cbind_input),
            self.nonce,
            STANDARD.encode(self.proof)
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum ServerFinalMessage {
    Success { verifier: Bytes },
    Error { message: String },
}

impl std::str::FromStr for ServerFinalMessage {
    type Err = SaslError;

    fn from_str(s: &str) -> Result<ServerFinalMessage, Self::Err> {
        let mut ss = s;
        match parse_gs2_pair(&mut ss)? {
            Some(("e", value)) => Ok(ServerFinalMessage::Error {
                message: value.to_owned(),
            }),
            Some(("v", b64)) => {
                let verifier = Bytes::from(STANDARD.decode(b64)?);
                Ok(ServerFinalMessage::Success { verifier })
            }
            _ => Err(SaslError::Parse),
        }
    }
}

type AuthzId = Nickname;

#[derive(Clone, Debug, Eq, PartialEq)]
struct Username(String);

impl Username {
    fn as_str(&self) -> &str {
        &self.0
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

#[derive(Clone, Debug, Eq, PartialEq)]
struct Password(String);

impl Password {
    fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl std::str::FromStr for Password {
    type Err = SaslError;

    fn from_str(s: &str) -> Result<Password, SaslError> {
        let s = stringprep::saslprep(s).map_err(SaslError::PreparePassword)?;
        Ok(Password(s.into_owned()))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Gs2Escaped<'a>(&'a str);

impl fmt::Display for Gs2Escaped<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for ch in self.0.chars() {
            match ch {
                ',' => write!(f, "=2C")?,
                '=' => write!(f, "=3D")?,
                c => write!(f, "{c}")?,
            }
        }
        Ok(())
    }
}

fn generate_nonce() -> String {
    let mut rng = StdRng::from_os_rng();
    Alphanumeric
        .sample_iter(&mut rng)
        .take(CLIENT_NONCE_LENGTH)
        .map(char::from)
        .collect()
}

fn parse_gs2_pair<'a>(s: &mut &'a str) -> Result<Option<(&'a str, &'a str)>, SaslError> {
    if s.is_empty() {
        return Ok(None);
    }
    let kv = match s.split_once(',') {
        Some((pre, post)) => {
            *s = post;
            pre
        }
        None => std::mem::take(s),
    };
    match kv.split_once('=') {
        Some((k, v)) => Ok(Some((k, v))),
        None => Err(SaslError::Parse),
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct Computation {
    client_proof: Bytes,
    server_signature: Bytes,
}

fn compute_scram(
    hash: HashAlgo,
    password: &Password,
    salt: &[u8],
    iteration_count: u32,
    auth_message: &str,
) -> Computation {
    let salted_password = hash.iter_hash(password.as_bytes(), salt, iteration_count);
    let client_key = hash.hmac(&salted_password, b"Client Key");
    let stored_key = hash.hash(&client_key);
    let client_signature = hash.hmac(&stored_key, auth_message.as_bytes());
    let client_proof = std::iter::zip(client_key, client_signature)
        .map(|(a, b)| a ^ b)
        .collect::<Bytes>();
    let server_key = hash.hmac(&salted_password, b"Server Key");
    let server_signature = hash.hmac(&server_key, auth_message.as_bytes());
    Computation {
        client_proof,
        server_signature,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod server_first_message {
        use super::*;

        #[test]
        fn parse_rfc_example() {
            let msg = "r=fyko+d2lbbFgONRv9qkxdawL3rfcNHYJY1ZVvWVs7j,s=QSXCR+Q6sek8bf92,i=4096"
                .parse::<ServerFirstMessage>()
                .unwrap();
            assert_eq!(
                msg,
                ServerFirstMessage {
                    nonce: String::from("fyko+d2lbbFgONRv9qkxdawL3rfcNHYJY1ZVvWVs7j"),
                    salt: Bytes::from(
                        b"\x41\x25\xc2\x47\xe4\x3a\xb1\xe9\x3c\x6d\xff\x76".as_slice()
                    ),
                    iteration_count: 4096,
                }
            );
        }

        #[test]
        fn parse_ircv3_example() {
            let msg = "r=c5RqLCZy0L4fGkKAZ0hujFBsXQoKcivqCw9iDZPSpb,s=5mJO6d4rjCnsBU1X,i=4096"
                .parse::<ServerFirstMessage>()
                .unwrap();
            assert_eq!(
                msg,
                ServerFirstMessage {
                    nonce: String::from("c5RqLCZy0L4fGkKAZ0hujFBsXQoKcivqCw9iDZPSpb"),
                    salt: Bytes::from(
                        b"\xe6\x62\x4e\xe9\xde\x2b\x8c\x29\xec\x05\x4d\x57".as_slice()
                    ),
                    iteration_count: 4096,
                }
            );
        }
    }

    mod server_final_message {
        use super::*;

        #[test]
        fn parse_rfc_example() {
            let msg = "v=rmF9pqV8S7suAoZWja4dJRkFsKQ="
                .parse::<ServerFinalMessage>()
                .unwrap();
            assert_eq!(
                msg,
                ServerFinalMessage::Success {
                    verifier: Bytes::from(
                        b"\xae\x61\x7d\xa6\xa5\x7c\x4b\xbb\x2e\x02\x86\x56\x8d\xae\x1d\x25\x19\x05\xb0\xa4".as_slice()
                    )
                }
            );
        }

        #[test]
        fn parse_ircv3_example() {
            let msg = "v=ZWR23c9MJir0ZgfGf5jEtLOn6Ng="
                .parse::<ServerFinalMessage>()
                .unwrap();
            assert_eq!(
                msg,
                ServerFinalMessage::Success {
                    verifier: Bytes::from(
                        b"\x65\x64\x76\xdd\xcf\x4c\x26\x2a\xf4\x66\x07\xc6\x7f\x98\xc4\xb4\xb3\xa7\xe8\xd8".as_slice()
                    )
                }
            );
        }

        #[test]
        fn parse_error() {
            let msg = "e=other-error".parse::<ServerFinalMessage>().unwrap();
            assert_eq!(
                msg,
                ServerFinalMessage::Error {
                    message: String::from("other-error")
                }
            );
        }
    }
}
