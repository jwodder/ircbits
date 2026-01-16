[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`irctext` is a Rust library for working with IRC messages (parsing,
constructing, rendering, etc.) in which every type of message (both client
messages and replies) is represented by a dedicated type that only permits
values that conform to the specification at <https://modern.ircdocs.horse>.

Features
========

The `irctext` crate has the following optional features:

- `anstyle` — Enables converting formatting types to
  [`anstyle`](https://crates.io/crate/anstyle) types and rendering IRC-styled
  text with ANSI sequences

- `serde` — Enables serializing & deserializing most types with
  [`serde`](https://crates.io/crate/serde)
