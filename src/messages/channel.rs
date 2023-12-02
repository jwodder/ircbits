// <https://modern.ircdocs.horse/#channels>:
//
// > Apart from the requirement of the first character being a valid channel
// > type prefix character; the only restriction on a channel name is that it
// > may not contain any spaces `(' ', 0x20)`, a control G / `BELL` `('^G',
// > 0x07)`, or a comma `(',', 0x2C)` (which is used as a list item separator
// > by the protocol).
//
// Note that this implementation does not enforce the requirement on the first
// character, as the set of valid channel type prefixes varies from server to
// server, and we have chosen to implement the `JOIN 0` command by specifying a
// prefixless channel of "0" for the "JOIN" verb.  However, this implementation
// does require that channel names not start with a colon (':', 0x3A), which is
// necessary in order to be able to pass parameters after a channel parameter.
use thiserror::Error;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Channel(String);

common_string!(Channel, ChannelError);

impl TryFrom<String> for Channel {
    type Error = ChannelError;

    fn try_from(s: String) -> Result<Channel, ChannelError> {
        if s.is_empty() {
            Err(ChannelError::Empty)
        } else if s.starts_with(':') {
            Err(ChannelError::StartsWithColon)
        } else if s.contains(['\0', '\r', '\n', ' ', '\x07', ',']) {
            Err(ChannelError::BadCharacter)
        } else {
            Ok(Channel(s))
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
pub(crate) enum ChannelError {
    #[error("channels cannot be empty")]
    Empty,
    #[error("channels cannot start with a colon")]
    StartsWithColon,
    #[error("channels cannot contain NUL, CR, LF, SPACE, BELL, or comma")]
    BadCharacter,
}
