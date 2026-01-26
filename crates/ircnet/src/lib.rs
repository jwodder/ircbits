#![cfg_attr(docsrs, feature(doc_cfg))]
//! `ircnet` is a Rust library for connecting to IRC servers and sending &
//! receiving messages using message types defined in the [`irctext`][] crate.
//!
//! The library is divided into two top-level modules: `connect`, which
//! contains low-level facilities for creating a TCP connection (plain or TLS)
//! along with [codecs][] that enable sending & receiving different
//! representations of messages; and `client`, which provides a higher-level
//! `Client` type for connecting to a server and sending & receiving message
//! structs along with facilities for defining *autoresponders* to incoming
//! messages and *commands* that carry out exchanges involving multiple
//! messages.
//!
//! [`irctext`]: https://github.com/jwodder/ircbits/tree/master/crates/irctext
//! [codecs]: https://docs.rs/tokio-util/latest/tokio_util/codec/index.html
//!
//! Features
//! ========
//!
//! The `ircnet` crate has the following optional feature:
//!
//! - `serde` — Enables serializing & deserializing certain types with
//!   [`serde`]
pub mod client;
pub mod connect;
pub mod sasl;
