[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![CI Status](https://github.com/jwodder/ircbits/actions/workflows/test.yml/badge.svg)](https://github.com/jwodder/ircbits/actions/workflows/test.yml)
[![codecov.io](https://codecov.io/gh/jwodder/ircbits/branch/master/graph/badge.svg)](https://codecov.io/gh/jwodder/ircbits)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

This is a [Rust][] [workspace][] containing assorted packages for working with
[IRC][] messages and acting as an IRC client.  It was made primarily for
personal use, is not intended for general use, and will likely not be placed on
crates.io.

[Rust]: https://www.rust-lang.org
[workspace]: https://doc.rust-lang.org/cargo/reference/workspaces.html
[IRC]: https://en.wikipedia.org/wiki/IRC

The code endeavors to follow the spec at <https://modern.ircdocs.horse>
strictly, including the following points:

- Only the commands and replies documented in the spec are supported, and only
  when their parameters follow the documented formats (though numeric replies
  are allowed to have more parameters than documented).
    - Exceptions to the above, largely to acheive basic compatibility with some
      actual servers:
        - Replies with unknown numeric codes are converted to a catch-all type
        - The following nonstandard numeric replies are supported:
            - `RPL_STATSCONN` (250)
            - `ERR_INVALIDCAPCMD` (410), specified by the [Capability
              Negotation specification][cap]
        - The `<nick>` parameter of `RPL_TOPICWHOTIME` (333) is allowed to be
          of the form `<nick>!<user>@<host>` rather than just a nickname

- The only supported channel type prefixes are `#` and `&`.

- The only support channel membership prefixes are `~`, `&`, `@`, `%`, and `+`.

- Tags are currently not yet implemented (jwodder/ircbits#4).

[cap]: https://ircv3.net/specs/extensions/capability-negotiation.html
