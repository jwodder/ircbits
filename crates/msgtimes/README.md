[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`msgtimes` logs into an IRC network, joins a given set of channels, and then
runs indefinitely, outputing a timestamped JSON object for each `PRIVMSG` and
`NOTICE` message thereafter received.

Options
=======

- `-c FILE`, `--config FILE` — Read IRC network connection details from the
  given configuration file, which must follow [the common `ircbits` config
  format][cfg] [default: `ircbits.toml`]

- `-o PATH`, `--outfile PATH` — Append the output to the given path.  By
  default, output is written to the standard output.

- `-P NAME`, `--profile NAME` — Use the connection details under the given
  profile in the configuration file [default: `irc`]

- `--trace` - Emit log events for every message sent & received

[cfg]: https://github.com/jwodder/ircbits/blob/master/doc/config-file.md

Configuration
=============

In addition to connection details, `msgtimes` reads further configuration from
the `msgtimes` subtable of the selected profile table in the configuration
file.  This subtable contains the following fields:

- `away` (string; optional) — If this is set, the given string will be sent as
  the argument of an `AWAY` message immediately after logging in

- `channels` (nonempty list of strings; required) — the names of the channels
  to join

Output Format
=============

`msgtimes`'s output is in JSON Lines format, with each line representing an IRC
event as a JSON object with the following fields:

- `"timestamp"` — the time at which the event occurred as an RFC 3339 timestamp

- `"network"` — the name of the IRC network as passed to the `--profile` option

- `"channel"` — the channel targeted by the event, or `null` if the event was
  not specific to any channel

- `"event"` — the type of event that occurred:
    - `"joined"` – `msgtimes` successfully completed joining the given channel
    - `"message"` – A `PRIVMSG` or `NOTICE` was sent to the given channel
        - `PRIVMSG` and `NOTICE` messages sent directly to the `msgtimes`
          client are not recorded.
    - `"kicked"` — The `msgtimes` client was kicked from the given channel
        - If `msgtimes` is kicked from every channel it joined, it exits.
    - `"disconnected"` — The server closed the connection
    - `"error"` — An error occured communicating with the server
