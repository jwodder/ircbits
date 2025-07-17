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
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::WhoIsActually(r)),
        } => {
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
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::TopicWhoTime(r)),
        } => {
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
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::NamReply(r)),
        } => {
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
            source: Some(Source::Server(host)),
            payload: Payload::Reply(Reply::Unknown(r)),
        } => {
            assert_eq!(host, Host::Domain("weber.oftc.net"));
            assert_eq!(r.code, 42);
            assert_eq!(r.parameters, ["jwodder", "9J5AACK4S", "your unique ID"]);
        });
    }
}
