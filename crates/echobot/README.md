[![Project Status: Concept – Minimal or no implementation has been done yet, or the repository is only intended to be a limited example, demo, or proof-of-concept.](https://www.repostatus.org/badges/latest/concept.svg)](https://www.repostatus.org/#concept)
[![Minimum Supported Rust Version](https://img.shields.io/badge/MSRV-1.91-orange)](https://www.rust-lang.org)
[![MIT License](https://img.shields.io/github/license/jwodder/ircbits.svg)](https://opensource.org/licenses/MIT)

`echobot` is a simple IRC bot that sends back any messages sent to it after a
five-second delay.  Specifically:

- If the bot is in a channel to which a user sends a `PRIVMSG` of the form
  "`<nickname>: <message>`" where `<nickname>` is the bot's nickname (matched
  case-insensitively), then the bot will send `<message>` to the channel after
  the delay.

- If the bot is sent a `PRIVMSG` directly, it will echo back the entirety of
  the message to the sender after the delay.

The bot does not respond to `NOTICE` messages.

A running `echobot` instance can be shut down gracefully by sending it SIGINT
(Ctrl-C) or SIGTERM, which will cause it to send a `QUIT` message to the server
and then exit once the server closes the connection.  `echobot` will also shut
down if it is kicked from every channel it has joined.

Options
=======

- `-c FILE`, `--config FILE` — Read IRC network connection details from the
  given configuration file, which must follow [the common `ircbits` config
  format][cfg] [default: `ircbits.toml`]

- `-P NAME`, `--profile NAME` — Use the connection details under the given
  profile in the configuration file [default: `irc`]

- `--trace` - Emit log events for every message sent & received

[cfg]: https://github.com/jwodder/ircbits/blob/master/doc/config-file.md

Configuration
=============

In addition to connection details, `echobot` reads further configuration from
the `echobot` subtable of the selected profile table in the configuration file.
This subtable contains the following field:

- `channels` (list of strings; optional) — names of channels to join upon login
