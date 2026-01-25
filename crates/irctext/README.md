[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`irctext` is a Rust library for working with IRC messages (parsing,
constructing, rendering, etc.) in which every type of message (both client
messages and replies) is represented by a dedicated type that only permits
values that conform to the specification at <https://modern.ircdocs.horse> plus
supported IRCv3 extensions.

In particular:

- Only the commands and replies documented in the spec are supported, and only
  when their parameters follow the documented formats (though numeric replies
  are allowed to have more parameters than documented).
    - Exceptions to the above, largely to achieve basic compatibility with some
      actual servers:
        - Replies with unknown numeric codes are converted to a catch-all type
        - The following nonstandard numeric replies are supported:
            - `RPL_STATSCONN` (250)
            - `RPL_CHANNELURL` (328)
        - The following client messages and numeric replies specified by IRCv3
          extensions are supported:
            - `TAGMSG` ([Message Tags][])
            - `ERR_INVALIDCAPCMD` (410) ([Capability Negotiation][cap])
        - The `<nick>` parameter of `RPL_TOPICWHOTIME` (333) is allowed to be
          either just a nickname or a string of the form `<nick>!<user>@<host>`
        - The trailing comments parameter of `RPL_VERSION` (351) is optional

- The only supported channel type prefixes are `#` and `&`.

- The only support channel membership prefixes are `~`, `&`, `@`, `%`, and `+`.

[Message Tags]: https://ircv3.net/specs/extensions/message-tags
[cap]: https://ircv3.net/specs/extensions/capability-negotiation.html

Features
========

The `irctext` crate has the following optional features:

- `anstyle` — Enables converting formatting types to
  [`anstyle`](https://crates.io/crate/anstyle) types and rendering IRC-styled
  text with ANSI sequences

- `serde` — Enables serializing & deserializing various types with
  [`serde`](https://crates.io/crate/serde)
