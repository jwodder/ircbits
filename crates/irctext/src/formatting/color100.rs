// Reference: <https://modern.ircdocs.horse/formatting#color>
use thiserror::Error;

/// A color that can be used for the foreground or background of text in an IRC
/// client.  Colors are identified by numbers from 0 through 99.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Color100(u8);

impl Color100 {
    /// Color 0, corresponding to ANSI bright white (index 15)
    pub const WHITE: Color100 = Color100(0);

    /// Color 1, corresponding to ANSI black (index 0)
    pub const BLACK: Color100 = Color100(1);

    /// Color 2, corresponding to ANSI blue (index 4)
    pub const BLUE: Color100 = Color100(2);

    /// Color 3, corresponding to ANSI green (index 2)
    pub const GREEN: Color100 = Color100(3);

    /// Color 4, corresponding to ANSI bright red (9)
    ///
    /// Also called "light red" by some clients
    pub const RED: Color100 = Color100(4);

    /// Color 5, corresponding to ANSI red (1)
    ///
    /// Also called "red" by some clients
    pub const BROWN: Color100 = Color100(5);

    /// Color 6, corresponding to ANSI magenta (5)
    pub const MAGENTA: Color100 = Color100(6);

    /// Color 7, corresponding to ANSI yellow (3)
    pub const ORANGE: Color100 = Color100(7);

    /// Color 8, corresponding to ANSI bright yellow (11)
    pub const YELLOW: Color100 = Color100(8);

    /// Color 9, corresponding to ANSI bright green (index 10)
    pub const LIGHT_GREEN: Color100 = Color100(9);

    /// Color 10, corresponding to ANSI cyan (index 6)
    pub const CYAN: Color100 = Color100(10);

    /// Color 11, corresponding to ANSI cyan (index 14)
    pub const LIGHT_CYAN: Color100 = Color100(11);

    /// Color 12, corresponding to ANSI bright blue (index 12)
    pub const LIGHT_BLUE: Color100 = Color100(12);

    /// Color 13, corresponding to ANSI bright magenta (index 13)
    ///
    /// Also called "light magenta" by some clients
    pub const PINK: Color100 = Color100(13);

    /// Color 14, corresponding to ANSI bright black (index 8)
    pub const GREY: Color100 = Color100(14);

    /// Color 15, corresponding to ANSI white (index 7)
    pub const LIGHT_GREY: Color100 = Color100(15);

    /// Color 99, the default foreground/background color
    pub const DEFAULT: Color100 = Color100(99);

    /// Returns the corresponding color number in the ANSI 256-color palette.
    ///
    /// `Color::DEFAULT` (IRC color 99) maps to `None`.
    ///
    /// The ANSI colors for IRC colors 16 through 98 are taken from
    /// <https://modern.ircdocs.horse/formatting#colors-16-98>.
    // Note that the indices for colors 0 though 15 are not standardized and
    // may vary from what other clients use.
    pub fn to_ansi_index(self) -> Option<u8> {
        match self.0 {
            0 => Some(15),
            1 => Some(0),
            2 => Some(4),
            3 => Some(2),
            4 => Some(9),
            5 => Some(1),
            6 => Some(5),
            7 => Some(3),
            8 => Some(11),
            9 => Some(10),
            10 => Some(6),
            11 => Some(14),
            12 => Some(12),
            13 => Some(13),
            14 => Some(8),
            15 => Some(7),
            16 => Some(52),
            17 => Some(94),
            18 => Some(100),
            19 => Some(58),
            20 => Some(22),
            21 => Some(29),
            22 => Some(23),
            23 => Some(24),
            24 => Some(17),
            25 => Some(54),
            26 => Some(53),
            27 => Some(89),
            28 => Some(88),
            29 => Some(130),
            30 => Some(142),
            31 => Some(64),
            32 => Some(28),
            33 => Some(35),
            34 => Some(30),
            35 => Some(25),
            36 => Some(18),
            37 => Some(91),
            38 => Some(90),
            39 => Some(125),
            40 => Some(124),
            41 => Some(166),
            42 => Some(184),
            43 => Some(106),
            44 => Some(34),
            45 => Some(49),
            46 => Some(37),
            47 => Some(33),
            48 => Some(19),
            49 => Some(129),
            50 => Some(127),
            51 => Some(161),
            52 => Some(196),
            53 => Some(208),
            54 => Some(226),
            55 => Some(154),
            56 => Some(46),
            57 => Some(86),
            58 => Some(51),
            59 => Some(75),
            60 => Some(21),
            61 => Some(171),
            62 => Some(201),
            63 => Some(198),
            64 => Some(203),
            65 => Some(215),
            66 => Some(227),
            67 => Some(191),
            68 => Some(83),
            69 => Some(122),
            70 => Some(87),
            71 => Some(111),
            72 => Some(63),
            73 => Some(177),
            74 => Some(207),
            75 => Some(205),
            76 => Some(217),
            77 => Some(223),
            78 => Some(229),
            79 => Some(193),
            80 => Some(157),
            81 => Some(158),
            82 => Some(159),
            83 => Some(153),
            84 => Some(147),
            85 => Some(183),
            86 => Some(219),
            87 => Some(212),
            88 => Some(16),
            89 => Some(233),
            90 => Some(235),
            91 => Some(237),
            92 => Some(239),
            93 => Some(241),
            94 => Some(244),
            95 => Some(247),
            96 => Some(250),
            97 => Some(254),
            98 => Some(231),
            99 => None,
            _ => unreachable!(),
        }
    }

    pub fn try_from_ansi_index(index: u8) -> Option<Color100> {
        match index {
            0 => Some(Color100(1)),
            1 => Some(Color100(5)),
            2 => Some(Color100(3)),
            3 => Some(Color100(7)),
            4 => Some(Color100(2)),
            5 => Some(Color100(6)),
            6 => Some(Color100(10)),
            7 => Some(Color100(15)),
            8 => Some(Color100(14)),
            9 => Some(Color100(4)),
            10 => Some(Color100(9)),
            11 => Some(Color100(8)),
            12 => Some(Color100(12)),
            13 => Some(Color100(13)),
            14 => Some(Color100(11)),
            15 => Some(Color100(0)),
            16 => Some(Color100(88)),
            17 => Some(Color100(24)),
            18 => Some(Color100(36)),
            19 => Some(Color100(48)),
            21 => Some(Color100(60)),
            22 => Some(Color100(20)),
            23 => Some(Color100(22)),
            24 => Some(Color100(23)),
            25 => Some(Color100(35)),
            28 => Some(Color100(32)),
            29 => Some(Color100(21)),
            30 => Some(Color100(34)),
            33 => Some(Color100(47)),
            34 => Some(Color100(44)),
            35 => Some(Color100(33)),
            37 => Some(Color100(46)),
            46 => Some(Color100(56)),
            49 => Some(Color100(45)),
            51 => Some(Color100(58)),
            52 => Some(Color100(16)),
            53 => Some(Color100(26)),
            54 => Some(Color100(25)),
            58 => Some(Color100(19)),
            63 => Some(Color100(72)),
            64 => Some(Color100(31)),
            75 => Some(Color100(59)),
            83 => Some(Color100(68)),
            86 => Some(Color100(57)),
            87 => Some(Color100(70)),
            88 => Some(Color100(28)),
            89 => Some(Color100(27)),
            90 => Some(Color100(38)),
            91 => Some(Color100(37)),
            94 => Some(Color100(17)),
            100 => Some(Color100(18)),
            106 => Some(Color100(43)),
            111 => Some(Color100(71)),
            122 => Some(Color100(69)),
            124 => Some(Color100(40)),
            125 => Some(Color100(39)),
            127 => Some(Color100(50)),
            129 => Some(Color100(49)),
            130 => Some(Color100(29)),
            142 => Some(Color100(30)),
            147 => Some(Color100(84)),
            153 => Some(Color100(83)),
            154 => Some(Color100(55)),
            157 => Some(Color100(80)),
            158 => Some(Color100(81)),
            159 => Some(Color100(82)),
            161 => Some(Color100(51)),
            166 => Some(Color100(41)),
            171 => Some(Color100(61)),
            177 => Some(Color100(73)),
            183 => Some(Color100(85)),
            184 => Some(Color100(42)),
            191 => Some(Color100(67)),
            193 => Some(Color100(79)),
            196 => Some(Color100(52)),
            198 => Some(Color100(63)),
            201 => Some(Color100(62)),
            203 => Some(Color100(64)),
            205 => Some(Color100(75)),
            207 => Some(Color100(74)),
            208 => Some(Color100(53)),
            212 => Some(Color100(87)),
            215 => Some(Color100(65)),
            217 => Some(Color100(76)),
            219 => Some(Color100(86)),
            223 => Some(Color100(77)),
            226 => Some(Color100(54)),
            227 => Some(Color100(66)),
            229 => Some(Color100(78)),
            231 => Some(Color100(98)),
            233 => Some(Color100(89)),
            235 => Some(Color100(90)),
            237 => Some(Color100(91)),
            239 => Some(Color100(92)),
            241 => Some(Color100(93)),
            244 => Some(Color100(94)),
            247 => Some(Color100(95)),
            250 => Some(Color100(96)),
            254 => Some(Color100(97)),
            _ => None,
        }
    }

    #[cfg(feature = "anstyle")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
    pub fn to_anstyle(self) -> Option<anstyle::Ansi256Color> {
        self.to_ansi_index().map(anstyle::Ansi256Color)
    }
}

impl Default for Color100 {
    fn default() -> Color100 {
        Color100::DEFAULT
    }
}

impl TryFrom<u8> for Color100 {
    type Error = Color100TryFromIntError;

    fn try_from(value: u8) -> Result<Color100, Color100TryFromIntError> {
        if (0..=99).contains(&value) {
            Ok(Color100(value))
        } else {
            Err(Color100TryFromIntError)
        }
    }
}

impl From<Color100> for u8 {
    fn from(value: Color100) -> u8 {
        value.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Error, PartialEq)]
#[error("IRC color numbers must be from 0 to 99")]
pub struct Color100TryFromIntError;
