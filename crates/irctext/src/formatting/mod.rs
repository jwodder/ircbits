mod attributes;
mod color100;
mod rgbcolor;
pub use self::attributes::*;
pub use self::color100::*;
pub use self::rgbcolor::*;
use std::borrow::Cow;
use std::fmt::Write;
use std::ops::Range;

#[cfg(feature = "anstyle")]
use std::fmt;

const BOLD_CHAR: char = '\x02';
const ITALIC_CHAR: char = '\x1D';
const UNDERLINE_CHAR: char = '\x1F';
const STRIKETHROUGH_CHAR: char = '\x1E';
const MONOSPACE_CHAR: char = '\x11';
const COLOR_CHAR: char = '\x03';
const HEX_COLOR_CHAR: char = '\x04';
const REVERSE_CHAR: char = '\x16';
const RESET_CHAR: char = '\x0F';

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Color {
    Color100(Color100),
    Rgb(RgbColor),
}

impl Color {
    #[cfg(feature = "anstyle")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
    pub fn to_anstyle(self) -> Option<anstyle::Color> {
        match self {
            Color::Color100(c) => c.to_anstyle().map(anstyle::Color::from),
            Color::Rgb(c) => Some(anstyle::Color::from(c.to_anstyle())),
        }
    }
}

impl Default for Color {
    fn default() -> Color {
        Color::Color100(Color100::default())
    }
}

impl From<Color100> for Color {
    fn from(value: Color100) -> Color {
        Color::Color100(value)
    }
}

impl From<RgbColor> for Color {
    fn from(value: RgbColor) -> Color {
        Color::Rgb(value)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub struct Style {
    pub foreground: Color,
    pub background: Color,
    pub attributes: AttributeSet,
}

impl Style {
    pub fn is_plain(self) -> bool {
        self == Style::default()
    }

    pub fn is_spoiler(self) -> bool {
        self.foreground == self.background && self.foreground != Color::default()
    }
}

#[cfg(feature = "anstyle")]
#[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
impl From<Style> for anstyle::Style {
    fn from(style: Style) -> anstyle::Style {
        anstyle::Style::new()
            .fg_color(style.foreground.to_anstyle())
            .bg_color(style.background.to_anstyle())
            .effects(style.attributes.into())
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StyledSpan<'a> {
    pub style: Style,
    pub content: Cow<'a, str>,
}

#[cfg(feature = "anstyle")]
#[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
impl<'a> StyledSpan<'a> {
    pub fn render_ansi(&self) -> RenderStyledSpan<'_, 'a> {
        RenderStyledSpan(self)
    }
}

impl<'a> From<String> for StyledSpan<'a> {
    fn from(value: String) -> StyledSpan<'a> {
        StyledSpan {
            style: Style::default(),
            content: Cow::from(value),
        }
    }
}

impl<'a> From<&'a str> for StyledSpan<'a> {
    fn from(value: &'a str) -> StyledSpan<'a> {
        StyledSpan {
            style: Style::default(),
            content: Cow::from(value),
        }
    }
}

#[cfg(feature = "anstyle")]
#[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderStyledSpan<'a, 'b>(&'a StyledSpan<'b>);

#[cfg(feature = "anstyle")]
impl fmt::Display for RenderStyledSpan<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let style = anstyle::Style::from(self.0.style);
        write!(f, "{style}{}{style:#}", self.0.content)
    }
}

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct StyledLine<'a>(pub Vec<StyledSpan<'a>>);

impl<'a> StyledLine<'a> {
    pub fn parse(s: &'a str) -> StyledLine<'a> {
        let mut builder = StyledLineBuilder::new();
        let mut char_indices = s.char_indices().peekable();
        let iter = char_indices.by_ref();
        while let Some((i, ch)) = iter.next() {
            match ch {
                BOLD_CHAR => builder.toggle(Attribute::Bold, i),
                ITALIC_CHAR => builder.toggle(Attribute::Italic, i),
                UNDERLINE_CHAR => builder.toggle(Attribute::Underline, i),
                STRIKETHROUGH_CHAR => builder.toggle(Attribute::Strikethrough, i),
                MONOSPACE_CHAR => builder.toggle(Attribute::Monospace, i),
                COLOR_CHAR => {
                    if let Some(fg) = scan_color100(iter) {
                        builder.set_foreground(Color::from(fg), i);
                        let comma_index = iter.next_if(|&(_, ch)| ch == ',').map(|(i, _)| i);
                        if let Some(bg) = scan_color100(iter) {
                            builder.set_background(Color::from(bg), i);
                        } else if let Some(ci) = comma_index {
                            builder.push_char(ci);
                        }
                    } else {
                        builder.reset_colors(i);
                    }
                }
                HEX_COLOR_CHAR => {
                    if let Some(fg) = scan_rgbcolor(iter) {
                        builder.set_foreground(Color::from(fg), i);
                        let comma_index = iter.next_if(|&(_, ch)| ch == ',').map(|(i, _)| i);
                        if let Some(bg) = scan_rgbcolor(iter) {
                            builder.set_background(Color::from(bg), i);
                        } else if let Some(ci) = comma_index {
                            builder.push_char(ci);
                        }
                    } else {
                        builder.reset_colors(i);
                        // scan_rgbcolor() may have gobbled up a partial hex
                        // string, so register it.  (If the HEX_COLOR_CHAR was
                        // instead followed by another formatting character,
                        // the empty span will be filtered out by the builder.)
                        builder.push_char(i + 1);
                    }
                }
                REVERSE_CHAR => builder.toggle(Attribute::Reverse, i),
                RESET_CHAR => builder.reset(i),
                _ => builder.push_char(i),
            }
        }
        StyledLine::from_iter(builder.finish(s.len()).map(|(style, range)| StyledSpan {
            style,
            content: Cow::from(&s[range]),
        }))
    }

    pub fn format(&self) -> String {
        let mut s = String::new();
        let mut prev_style = Style::default();
        for span in self {
            if span.content.is_empty() {
                continue;
            }
            match StyleDiff::new(prev_style, span.style) {
                StyleDiff::Reset => s.push(RESET_CHAR),
                StyleDiff::Delta {
                    toggled,
                    color_diff,
                } => {
                    for attr in toggled {
                        match attr {
                            Attribute::Bold => s.push(BOLD_CHAR),
                            Attribute::Italic => s.push(ITALIC_CHAR),
                            Attribute::Underline => s.push(UNDERLINE_CHAR),
                            Attribute::Strikethrough => s.push(STRIKETHROUGH_CHAR),
                            Attribute::Monospace => s.push(MONOSPACE_CHAR),
                            Attribute::Reverse => s.push(REVERSE_CHAR),
                        }
                    }
                    match color_diff {
                        ColorDiff::NoChange => (),
                        ColorDiff::Reset => {
                            s.push(COLOR_CHAR);
                            if span.content.starts_with(|ch: char| ch.is_ascii_digit()) {
                                s.push(BOLD_CHAR);
                                s.push(BOLD_CHAR);
                            }
                        }
                        ColorDiff::SetFg(color) => match color {
                            Color::Color100(c) => {
                                write!(&mut s, "{COLOR_CHAR}{:02}", u8::from(c)).unwrap();
                                if span.content.starts_with(',')
                                    && span.content[1..].starts_with(|ch: char| ch.is_ascii_digit())
                                {
                                    s.push(BOLD_CHAR);
                                    s.push(BOLD_CHAR);
                                }
                            }
                            Color::Rgb(RgbColor(r, g, b)) => {
                                write!(&mut s, "{HEX_COLOR_CHAR}{r:02x}{g:02x}{b:02x}").unwrap();
                            }
                        },
                        ColorDiff::SetBoth { fg, bg } => match (fg, bg) {
                            (Color::Color100(fg), Color::Color100(bg)) => {
                                write!(
                                    &mut s,
                                    "{COLOR_CHAR}{:02},{:02}",
                                    u8::from(fg),
                                    u8::from(bg)
                                )
                                .unwrap();
                            }
                            (
                                Color::Rgb(RgbColor(r1, g1, b1)),
                                Color::Rgb(RgbColor(r2, g2, b2)),
                            ) => {
                                write!(
                                    &mut s,
                                    "{HEX_COLOR_CHAR}{r1:02x}{g1:02x}{b1:02x},{r2:02x}{g2:02x}{b2:02x}",
                                )
                                .unwrap();
                            }
                            (Color::Color100(fg), Color::Rgb(RgbColor(r, g, b))) => {
                                write!(
                                    &mut s,
                                    "{HEX_COLOR_CHAR}000000,{r:02x}{g:02x}{b:02x}{COLOR_CHAR}{:02}",
                                    u8::from(fg)
                                )
                                .unwrap();
                                if span.content.starts_with(',')
                                    && span.content[1..].starts_with(|ch: char| ch.is_ascii_digit())
                                {
                                    s.push(BOLD_CHAR);
                                    s.push(BOLD_CHAR);
                                }
                            }
                            (Color::Rgb(RgbColor(r, g, b)), Color::Color100(bg)) => {
                                write!(
                                    &mut s,
                                    "{COLOR_CHAR}99,{:02}{HEX_COLOR_CHAR}{r:02x}{g:02x}{b:02x}",
                                    u8::from(bg)
                                )
                                .unwrap();
                            }
                        },
                    }
                }
            }
            s.push_str(&span.content);
            prev_style = span.style;
        }
        s
    }

    pub fn iter<'b>(&'b self) -> StyledLineIter<'b, 'a> {
        self.into_iter()
    }

    #[cfg(feature = "anstyle")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
    pub fn render_ansi<'b>(&'b self) -> RenderStyledLine<'b, 'a> {
        RenderStyledLine(self)
    }
}

#[cfg(feature = "anstyle")]
#[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RenderStyledLine<'a, 'b>(&'a StyledLine<'b>);

#[cfg(feature = "anstyle")]
impl fmt::Display for RenderStyledLine<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for s in &self.0.0 {
            write!(f, "{}", s.render_ansi())?;
        }
        Ok(())
    }
}

impl<'a> From<StyledSpan<'a>> for StyledLine<'a> {
    fn from(value: StyledSpan<'a>) -> StyledLine<'a> {
        StyledLine(vec![value])
    }
}

impl<'a> FromIterator<StyledSpan<'a>> for StyledLine<'a> {
    fn from_iter<I: IntoIterator<Item = StyledSpan<'a>>>(iter: I) -> StyledLine<'a> {
        StyledLine(Vec::from_iter(iter))
    }
}

impl<'a> Extend<StyledSpan<'a>> for StyledLine<'a> {
    fn extend<I: IntoIterator<Item = StyledSpan<'a>>>(&mut self, iter: I) {
        self.0.extend(iter);
    }
}

impl<'a> IntoIterator for StyledLine<'a> {
    type IntoIter = StyledLineIntoIter<'a>;
    type Item = StyledSpan<'a>;

    fn into_iter(self) -> StyledLineIntoIter<'a> {
        StyledLineIntoIter(self.0.into_iter())
    }
}

impl<'a, 'b> IntoIterator for &'b StyledLine<'a> {
    type IntoIter = StyledLineIter<'b, 'a>;
    type Item = &'b StyledSpan<'a>;

    fn into_iter(self) -> StyledLineIter<'b, 'a> {
        StyledLineIter(self.0.iter())
    }
}

#[derive(Clone, Debug)]
pub struct StyledLineIntoIter<'a>(std::vec::IntoIter<StyledSpan<'a>>);

impl<'a> Iterator for StyledLineIntoIter<'a> {
    type Item = StyledSpan<'a>;

    fn next(&mut self) -> Option<StyledSpan<'a>> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a> DoubleEndedIterator for StyledLineIntoIter<'a> {
    fn next_back(&mut self) -> Option<StyledSpan<'a>> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for StyledLineIntoIter<'_> {}

impl std::iter::FusedIterator for StyledLineIntoIter<'_> {}

#[derive(Clone, Debug)]
pub struct StyledLineIter<'a, 'b>(std::slice::Iter<'a, StyledSpan<'b>>);

impl<'a, 'b> Iterator for StyledLineIter<'a, 'b> {
    type Item = &'a StyledSpan<'b>;

    fn next(&mut self) -> Option<&'a StyledSpan<'b>> {
        self.0.next()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

impl<'a, 'b> DoubleEndedIterator for StyledLineIter<'a, 'b> {
    fn next_back(&mut self) -> Option<&'a StyledSpan<'b>> {
        self.0.next_back()
    }
}

impl ExactSizeIterator for StyledLineIter<'_, '_> {}

impl std::iter::FusedIterator for StyledLineIter<'_, '_> {}

#[derive(Clone, Debug, Eq, PartialEq)]
struct StyledLineBuilder {
    closed: Vec<(Style, Range<usize>)>,
    open: OpenStyledSpan,
}

impl StyledLineBuilder {
    fn new() -> StyledLineBuilder {
        StyledLineBuilder {
            closed: Vec::new(),
            open: OpenStyledSpan::Styling(Style::default()),
        }
    }

    fn get_open_style(&mut self, index: usize) -> &mut Style {
        loop {
            match self.open {
                OpenStyledSpan::Styling(ref mut style) => return style,
                span @ OpenStyledSpan::Spanning { style, .. } => {
                    self.closed.extend(span.close(index));
                    self.open = OpenStyledSpan::Styling(style);
                }
            }
        }
    }

    fn toggle(&mut self, attr: Attribute, index: usize) {
        self.get_open_style(index).attributes ^= attr;
    }

    fn set_foreground(&mut self, color: Color, index: usize) {
        self.get_open_style(index).foreground = color;
    }

    fn set_background(&mut self, color: Color, index: usize) {
        self.get_open_style(index).background = color;
    }

    fn reset(&mut self, index: usize) {
        *self.get_open_style(index) = Style::default();
    }

    fn reset_colors(&mut self, index: usize) {
        self.set_foreground(Color::default(), index);
        self.set_background(Color::default(), index);
    }

    fn push_char(&mut self, index: usize) {
        if let OpenStyledSpan::Styling(style) = self.open {
            self.open = OpenStyledSpan::Spanning {
                style,
                start: index,
            };
        }
    }

    fn finish(mut self, str_len: usize) -> std::vec::IntoIter<(Style, Range<usize>)> {
        self.closed.extend(self.open.close(str_len));
        self.closed.into_iter()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum OpenStyledSpan {
    Styling(Style),
    Spanning { style: Style, start: usize },
}

impl OpenStyledSpan {
    fn close(self, end: usize) -> Option<(Style, Range<usize>)> {
        match self {
            OpenStyledSpan::Spanning { style, start } if start < end => Some((style, start..end)),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum StyleDiff {
    Delta {
        toggled: AttributeSet,
        color_diff: ColorDiff,
    },
    Reset,
}

impl StyleDiff {
    fn new(before: Style, after: Style) -> StyleDiff {
        if before == after {
            StyleDiff::Delta {
                toggled: AttributeSet::EMPTY,
                color_diff: ColorDiff::NoChange,
            }
        } else if after.is_plain() {
            StyleDiff::Reset
        } else {
            let mut toggled = AttributeSet::new();
            for attr in Attribute::iter() {
                if before.attributes.contains(attr) != after.attributes.contains(attr) {
                    toggled |= attr;
                }
            }
            let color_diff =
                if after.foreground == Color::default() && after.background == Color::default() {
                    ColorDiff::Reset
                } else {
                    match (
                        before.foreground == after.foreground,
                        before.background == after.background,
                    ) {
                        (true, true) => ColorDiff::NoChange,
                        (false, true) => ColorDiff::SetFg(after.foreground),
                        (_, false) => ColorDiff::SetBoth {
                            fg: after.foreground,
                            bg: after.background,
                        },
                    }
                };
            StyleDiff::Delta {
                toggled,
                color_diff,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ColorDiff {
    NoChange,
    Reset,
    SetFg(Color),
    SetBoth { fg: Color, bg: Color },
}

fn scan_color100<I>(iter: &mut std::iter::Peekable<I>) -> Option<Color100>
where
    I: Iterator<Item = (usize, char)>,
{
    let d1 = iter
        .next_if(|(_, ch)| ch.is_ascii_digit())
        .map(|(_, ch)| ch)
        .and_then(|ch| ch.to_digit(10))
        .and_then(|d| u8::try_from(d).ok());
    let d2 = iter
        .next_if(|(_, ch)| ch.is_ascii_digit())
        .map(|(_, ch)| ch)
        .and_then(|ch| ch.to_digit(10))
        .and_then(|d| u8::try_from(d).ok());
    let index = match (d1, d2) {
        (Some(d1), None) => d1,
        (Some(d1), Some(d2)) => d1 * 10 + d2,
        (None, None) => return None,
        (None, Some(_)) => unreachable!(),
    };
    Color100::try_from(index).ok()
}

fn scan_rgbcolor<I>(iter: &mut std::iter::Peekable<I>) -> Option<RgbColor>
where
    I: Iterator<Item = (usize, char)>,
{
    let d1 = scan_hexdigit(iter)?;
    let d2 = scan_hexdigit(iter)?;
    let d3 = scan_hexdigit(iter)?;
    let d4 = scan_hexdigit(iter)?;
    let d5 = scan_hexdigit(iter)?;
    let d6 = scan_hexdigit(iter)?;
    let r = d1 * 16 + d2;
    let g = d3 * 16 + d4;
    let b = d5 * 16 + d6;
    Some(RgbColor(r, g, b))
}

#[allow(clippy::return_and_then)]
fn scan_hexdigit<I>(iter: &mut std::iter::Peekable<I>) -> Option<u8>
where
    I: Iterator<Item = (usize, char)>,
{
    iter.next_if(|(_, ch)| ch.is_ascii_hexdigit())
        .map(|(_, ch)| ch)
        .and_then(|ch| ch.to_digit(16))
        .and_then(|d| u8::try_from(d).ok())
}

#[cfg(test)]
mod tests {
    use super::*;

    mod examples {
        use super::*;

        // Examples taken from <https://modern.ircdocs.horse/formatting#examples>

        #[test]
        fn example1() {
            let s = "I love \x033IRC! \x03It is the \x037best protocol ever!";
            let sline = StyledLine::parse(s);
            assert_eq!(
                sline,
                StyledLine(vec![
                    StyledSpan {
                        style: Style::default(),
                        content: "I love ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            foreground: Color100::GREEN.into(),
                            ..Style::default()
                        },
                        content: "IRC! ".into(),
                    },
                    StyledSpan {
                        style: Style::default(),
                        content: "It is the ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            foreground: Color100::ORANGE.into(),
                            ..Style::default()
                        },
                        content: "best protocol ever!".into(),
                    },
                ])
            );
        }

        #[test]
        fn example2() {
            let s = "This is a \x1D\x0313,9cool \x03message";
            let sline = StyledLine::parse(s);
            assert_eq!(
                sline,
                StyledLine(vec![
                    StyledSpan {
                        style: Style::default(),
                        content: "This is a ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            foreground: Color100::PINK.into(),
                            background: Color100::LIGHT_GREEN.into(),
                            attributes: Attribute::Italic.into()
                        },
                        content: "cool ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            attributes: Attribute::Italic.into(),
                            ..Style::default()
                        },
                        content: "message".into(),
                    }
                ])
            );
        }

        #[test]
        fn example3() {
            let s = "IRC \x02is \x034,12so \x03great\x0F!";
            let sline = StyledLine::parse(s);
            assert_eq!(
                sline,
                StyledLine(vec![
                    StyledSpan {
                        style: Style::default(),
                        content: "IRC ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            attributes: Attribute::Bold.into(),
                            ..Style::default()
                        },
                        content: "is ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            foreground: Color100::RED.into(),
                            background: Color100::LIGHT_BLUE.into(),
                            attributes: Attribute::Bold.into(),
                        },
                        content: "so ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            attributes: Attribute::Bold.into(),
                            ..Style::default()
                        },
                        content: "great".into(),
                    },
                    StyledSpan {
                        style: Style::default(),
                        content: "!".into()
                    },
                ])
            );
        }

        #[test]
        fn example4() {
            let s = "Rules: Don't spam 5\x0313,8,6\x03,7,8, and especially not \x029\x02\x1D!";
            let sline = StyledLine::parse(s);
            assert_eq!(
                sline,
                StyledLine(vec![
                    StyledSpan {
                        style: Style::default(),
                        content: "Rules: Don't spam 5".into(),
                    },
                    StyledSpan {
                        style: Style {
                            foreground: Color100::PINK.into(),
                            background: Color100::YELLOW.into(),
                            ..Style::default()
                        },
                        content: ",6".into(),
                    },
                    StyledSpan {
                        style: Style::default(),
                        content: ",7,8, and especially not ".into(),
                    },
                    StyledSpan {
                        style: Style {
                            attributes: Attribute::Bold.into(),
                            ..Style::default()
                        },
                        content: "9".into(),
                    },
                    StyledSpan {
                        style: Style {
                            attributes: Attribute::Italic.into(),
                            ..Style::default()
                        },
                        content: "!".into(),
                    },
                ])
            );
        }
    }

    #[test]
    fn color_comma_end() {
        let s = "\x03,";
        let sline = StyledLine::parse(s);
        assert_eq!(sline, StyledLine::from(StyledSpan::from(",")));
    }

    #[test]
    fn color_comma_not_digit() {
        let s = "\x034,a";
        let sline = StyledLine::parse(s);
        assert_eq!(
            sline,
            StyledLine(vec![StyledSpan {
                style: Style {
                    foreground: Color100::RED.into(),
                    ..Style::default()
                },
                content: ",a".into(),
            }])
        );
    }

    #[test]
    fn short_hex() {
        let s = "\x04ff00glarch";
        let sline = StyledLine::parse(s);
        assert_eq!(sline, StyledLine::from(StyledSpan::from("ff00glarch")));
    }

    #[test]
    fn hex_char_other_fmt() {
        let s = "\x04\x02foo";
        let sline = StyledLine::parse(s);
        assert_eq!(
            sline,
            StyledLine::from(StyledSpan {
                style: Style {
                    attributes: Attribute::Bold.into(),
                    ..Style::default()
                },
                content: "foo".into()
            })
        );
    }

    mod roundtrip {
        use super::*;

        #[test]
        fn colored_text_reset_colors_digits() {
            let sline = StyledLine(vec![
                StyledSpan {
                    style: Style {
                        foreground: Color100::BLUE.into(),
                        background: Color100::WHITE.into(),
                        attributes: Attribute::Bold.into(),
                    },
                    content: "Cloudy!".into(),
                },
                StyledSpan {
                    style: Style {
                        attributes: Attribute::Bold.into(),
                        ..Style::default()
                    },
                    content: "12345".into(),
                },
            ]);
            assert_eq!(StyledLine::parse(&sline.format()), sline);
        }

        #[test]
        fn foreground_comma_digits() {
            let sline = StyledLine::from(StyledSpan {
                style: Style {
                    foreground: Color100::GREEN.into(),
                    ..Style::default()
                },
                content: ",123".into(),
            });
            assert_eq!(StyledLine::parse(&sline.format()), sline);
        }

        #[test]
        fn color100_fg_rgb_bg() {
            let sline = StyledLine::from(StyledSpan {
                style: Style {
                    foreground: Color100::YELLOW.into(),
                    background: RgbColor(0x36, 0x36, 0x36).into(),
                    ..Style::default()
                },
                content: "My eyes hurt.".into(),
            });
            assert_eq!(StyledLine::parse(&sline.format()), sline);
        }

        #[test]
        fn rgb_fg_color100_bg() {
            let sline = StyledLine::from(StyledSpan {
                style: Style {
                    foreground: RgbColor(0x36, 0x36, 0x36).into(),
                    background: Color100::YELLOW.into(),
                    ..Style::default()
                },
                content: "My eyes hurt.".into(),
            });
            assert_eq!(StyledLine::parse(&sline.format()), sline);
        }
    }
}
