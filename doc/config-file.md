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
- `port` (integer; optional) — the port of the remote host to connect to;
  defaults to 6667 when `tls` is `false` and to 6697 when `tls` is `true`
- `tls` (boolean; optional; default: `true`) — whether to use SSL/TLS
  encryption for the connection
- `password` (string) — the password to log into IRC with
- `nickname` (string) — the nickname to use when logging into IRC
- `username` (string) — the username to declare when logging into IRC
- `realname` (string) — the "real name" to declare when logging into IRC
- `sasl` (boolean; optional; default: `true`) — Whether to attempt to
  authenticate using SASL.  If this is set to `true` and the remote server does
  not support capability negotiation or SASL, the program will proceed normally
  without using SASL.
- `sasl-mechanisms` (nonempty list of strings; optional) — The list of SASL
  mechanisms to use as part of authenticating with SASL, in decreasing order of
  preference.  The supported SASL mechanisms in their default order is:

    - `"SCRAM-SHA-512"`
    - `"SCRAM-SHA-256"`
    - `"SCRAM-SHA-1"`
    - `"PLAIN"`

    If a server does not support any configured mechanisms, the program will
    proceed normally without using SASL.

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
nickname = "edsample"
username = "edsample"
realname = "Edward Q. Sample"
password = "hunter3"
sasl = false
```
