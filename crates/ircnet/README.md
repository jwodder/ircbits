[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`ircnet` is a Rust library for connecting to IRC servers and sending &
receiving messages using message types defined in the [`irctext`][] crate.

The library is divided into two top-level modules: `connect`, which contains
low-level facilities for creating a TCP connection (plain or TLS) along with
[codecs][] that enable sending & receiving different representations of
messages; and `client`, which provides a higher-level `Client` type for
connecting to a server and sending & receiving message structs along with
facilities for defining *autoresponders* to incoming messages and *commands*
that carry out exchanges involving multiple messages.

[`irctext`]: https://github.com/jwodder/ircbits/tree/master/crates/irctext
[codecs]: https://docs.rs/tokio-util/latest/tokio_util/codec/index.html

Features
========

The `ircnet` crate has the following optional feature:

- `serde` — Enables serializing & deserializing certain types with
  [`serde`](https://crates.io/crate/serde)
