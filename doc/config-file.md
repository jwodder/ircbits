`ircbits` Configuration File Format
===================================

The various programs in the `ircbits` suite all read IRC connection details and
other settings from TOML configuration files that follow a common schema.  An
`ircbits` configuration file consists of a number of top-level TOML tables that
each define a *profile* containing the connection & login details for an
individual IRC network.  Which profile a command should use can be specified on
the command line via the `--profile` option, which takes the name of the
desired profile table; when this option is not given, all commands default to
looking for & using a profile named "irc".

A profile table contains the following fields, all of which are required unless
stated otherwise.

- `host` (string) — the hostname to connect to in order to access the IRC
  network
- `port` (integer) — the port of the remote host to connect to
- `tls` (boolean) — whether to use SSL/TLS encryption for the connection
- `password` (string) — the password to log into IRC with
- `nickname` (string) — the nickname to use when logging into IRC
- `username` (string) — the username to declare when logging into IRC
- `realname` (string) — the "real name" to declare when logging into IRC
- `sasl` (boolean; optional; default: `true`) — Whether to attempt to
  authenticate using SASL.  If this is set to `true` and the remote server does
  not support capability negotiation, the program will proceed normally without
  using SASL.

Many `ircbits` commands read further command-specific configuration from
subtables of profile tables that are keyed by the name of the command; see each
command's documentation for details.

Example
-------

The following configuration file defines two profiles, one for Libera and one
for OFTC.  The Libera profile contains further configuration specific to the
`ircwatch` command.

```toml
[libera]
host = "irc.libera.chat"
port = 6697
tls = true
nickname = "example"
username = "edsample"
realname = "Edward Sample"
password = "hunter2"

[libera.ircwatch]
default-channels = ["#python", "##rust", "#nethack"]

[oftc]
host = "irc.oftc.net"
port = 6697
tls = true
nickname = "edsample"
username = "edsample"
realname = "Edward Q. Sample"
password = "hunter2"
```
