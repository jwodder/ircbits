/// Enum of channel membership types
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ChannelMembership {
    Founder,
    Protected,
    Operator,
    HalfOperator,
    Voiced,
}

impl ChannelMembership {
    pub fn as_prefix(self) -> char {
        match self {
            ChannelMembership::Founder => '~',
            ChannelMembership::Protected => '&',
            ChannelMembership::Operator => '@',
            ChannelMembership::HalfOperator => '%',
            ChannelMembership::Voiced => '+',
        }
    }

    pub fn from_prefix(ch: char) -> Option<ChannelMembership> {
        match ch {
            '~' => Some(ChannelMembership::Founder),
            '&' => Some(ChannelMembership::Protected),
            '@' => Some(ChannelMembership::Operator),
            '%' => Some(ChannelMembership::HalfOperator),
            '+' => Some(ChannelMembership::Voiced),
            _ => None,
        }
    }

    pub fn as_mode(&self) -> char {
        match self {
            ChannelMembership::Founder => 'q',
            ChannelMembership::Protected => 'a',
            ChannelMembership::Operator => 'o',
            ChannelMembership::HalfOperator => 'h',
            ChannelMembership::Voiced => 'v',
        }
    }

    pub fn from_mode(ch: char) -> Option<ChannelMembership> {
        match ch {
            'q' => Some(ChannelMembership::Founder),
            'a' => Some(ChannelMembership::Protected),
            'o' => Some(ChannelMembership::Operator),
            'h' => Some(ChannelMembership::HalfOperator),
            'v' => Some(ChannelMembership::Voiced),
            _ => None,
        }
    }
}
