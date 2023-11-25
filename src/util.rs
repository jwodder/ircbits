pub(crate) fn decode_utf8_latin1(bs: Vec<u8>) -> String {
    match String::from_utf8(bs) {
        Ok(s) => s,
        Err(e) => e.into_bytes().into_iter().map(char::from).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_utf8latin1_good() {
        let bs = b"Snow\xC3\xA9mon: \xE2\x98\x83!".to_vec();
        assert_eq!(decode_utf8_latin1(bs), "Snowémon: ☃!");
    }

    #[test]
    fn test_decode_utf8latin1_fallback() {
        let bs = b"Snow\xC3\xA9mon: \xE2\x98!".to_vec();
        assert_eq!(decode_utf8_latin1(bs), "Snow\u{c3}\u{a9}mon: \u{e2}\u{98}!");
    }
}
