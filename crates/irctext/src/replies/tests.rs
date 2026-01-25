use super::*;
use crate::Source;
use assert_matches::assert_matches;
use std::net::Ipv4Addr;
use url::Host;

mod whoisactually {
    use super::*;

    #[test]
    fn just_ip_host() {
        let msg = ":molybdenum.libera.chat 338 jwodder somenick 127.0.0.1 :actually using host";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::WhoIsActually(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("molybdenum.libera.chat"));
            assert_eq!(r.client(), "jwodder");
            assert_eq!(r.nickname(), "somenick");
            assert_eq!(r.host(), None);
            assert_eq!(r.username(), None);
            assert_eq!(r.ip(), Some(IpAddr::V4(Ipv4Addr::LOCALHOST)));
            assert_eq!(r.message(), "actually using host");
        });
    }

    #[test]
    fn topicwhotime_libera() {
        let msg =
            ":calcium.libera.chat 333 jwodder #python nedbat!~nedbat@python/psf/nedbat 1698253660";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::TopicWhoTime(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("calcium.libera.chat"));
            assert_eq!(r.client(), "jwodder");
            assert_eq!(r.channel(), "#python");
            assert_eq!(r.user().nickname, "nedbat");
            assert_eq!(r.user().user.as_ref().unwrap(), "~nedbat");
            assert_eq!(r.user().host.as_ref().unwrap(), "python/psf/nedbat");
            assert_eq!(r.setat(), 1698253660);
        });
    }

    #[test]
    fn namreply_with_prefix() {
        let msg = ":silver.libera.chat 353 jwodder = #python :dostoyevsky2 tk @litharge snowolf DarthOreo enyc_ `Nothing4You fizi[DWBouncers]";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::NamReply(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("silver.libera.chat"));
            assert_eq!(r.client(), "jwodder");
            assert_eq!(r.channel_status(), ChannelStatus::Public);
            assert_eq!(r.channel(), "#python");
            assert_eq!(r.clients(), [
                (None, "dostoyevsky2".parse::<Nickname>().unwrap()),
                (None, "tk".parse::<Nickname>().unwrap()),
                (Some(ChannelMembership::Operator), "litharge".parse::<Nickname>().unwrap()),
                (None, "snowolf".parse::<Nickname>().unwrap()),
                (None, "DarthOreo".parse::<Nickname>().unwrap()),
                (None, "enyc_".parse::<Nickname>().unwrap()),
                (None, "`Nothing4You".parse::<Nickname>().unwrap()),
                (None, "fizi[DWBouncers]".parse::<Nickname>().unwrap()),
            ]);
        });
    }

    #[test]
    fn unknown_042() {
        let msg = ":weber.oftc.net 042 jwodder 9J5AACK4S :your unique ID";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::Unknown(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("weber.oftc.net"));
            assert_eq!(r.code, 42);
            assert_eq!(r.parameters, ["jwodder", "9J5AACK4S", "your unique ID"]);
        });
    }

    #[test]
    fn loggedin_with_star_source() {
        let msg = ":irc.ergo.chat 900 * * jwodder :You are now logged in as jwodder";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::LoggedIn(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("irc.ergo.chat"));
            assert_eq!(r.code(), 900);
            assert_eq!(r.client(), "*");
            assert!(r.your_source().is_none());
            assert_eq!(r.account(), "jwodder");
            assert_eq!(r.message(), "You are now logged in as jwodder");
            assert_eq!(r.parameters, ["*", "*", "jwodder", "You are now logged in as jwodder"]);
        });
    }

    #[test]
    fn isupport_draft() {
        // <https://ircv3.net/specs/extensions/network-icon>
        let msg = ":irc.example.org 005 * NETWORK=Example draft/ICON=https://example.org/icon.svg :are supported by this server";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::ISupport(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("irc.example.org"));
            assert_eq!(r.code(), 5);
            assert_eq!(r.client(), "*");
            assert_matches!(r.tokens(), [p1, p2] => {
                assert_matches!(p1, ISupportParam::Eq(k, v) => {
                    assert_eq!(k, "NETWORK");
                    assert_eq!(v, "Example");
                });
                assert_matches!(p2, ISupportParam::Eq(k, v) => {
                    assert_eq!(k, "draft/ICON");
                    assert_eq!(v, "https://example.org/icon.svg");
                });
            });
            assert_eq!(r.message(), "are supported by this server");
            assert_eq!(r.parameters, ["*", "NETWORK=Example", "draft/ICON=https://example.org/icon.svg", "are supported by this server"]);
        });
    }

    #[test]
    fn channelurl() {
        let msg = ":services. 328 jwodder #irssi :irssi.org";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::ChannelUrl(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("services."));
            assert_eq!(r.code(), 328);
            assert_eq!(r.client(), "jwodder");
            assert_eq!(r.channel(), "#irssi");
            assert_eq!(r.url(), "irssi.org");
            assert_eq!(r.parameters, ["jwodder", "#irssi", "irssi.org"]);
        });
    }

    #[test]
    fn umodeis() {
        let msg = ":molybdenum.libera.chat 221 jwodder +Ziw";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::UModeIs(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("molybdenum.libera.chat"));
            assert_eq!(r.code(), 221);
            assert_eq!(r.client(), "jwodder");
            assert_eq!(r.user_modes(), "+Ziw");
            assert_eq!(r.parameters, ["jwodder", "+Ziw"]);
        });
    }

    #[test]
    fn version_no_comment() {
        let msg = ":irc.ergo.chat 351 jwodder ergo-v2.17.0-rc1 irc.ergo.chat";
        let msg = msg.parse::<Message>().unwrap();
        assert_matches!(msg, Message {
            tags,
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::Version(r)),
        } => {
            assert!(tags.is_empty());
            assert_eq!(host, Host::Domain("irc.ergo.chat"));
            assert_eq!(r.code(), 351);
            assert_eq!(r.client(), "jwodder");
            assert_eq!(r.version(), "ergo-v2.17.0-rc1");
            assert_eq!(r.server(), "irc.ergo.chat");
            assert!(r.comments().is_none());
            assert_eq!(r.parameters, ["jwodder", "ergo-v2.17.0-rc1", "irc.ergo.chat"]);
        });
    }
}
