pub const PLAIN_PORT: u16 = 6667;

pub const TLS_PORT: u16 = 6697;

// Both RFC 2812 and <https://modern.ircdocs.horse> say that IRC messages (when
// tags aren't involved) are limited to 512 characters, counting the CR LF.
pub const MAX_LINE_LENGTH: usize = 512;

pub const MAX_LINE_LENGTH_WITH_TAGS: usize = MAX_LINE_LENGTH + 8191;
