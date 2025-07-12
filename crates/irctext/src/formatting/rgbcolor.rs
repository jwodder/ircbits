/// A 24-bit color composed of red, green, and blue components
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct RgbColor(
    /// Red
    pub u8,
    /// Green
    pub u8,
    /// Blue
    pub u8,
);

impl RgbColor {
    /// Return the red component
    pub fn red(self) -> u8 {
        self.0
    }

    /// Return the green component
    pub fn green(self) -> u8 {
        self.1
    }

    /// Return the blue component
    pub fn blue(self) -> u8 {
        self.2
    }

    #[cfg(feature = "anstyle")]
    #[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
    pub fn to_anstyle(self) -> anstyle::RgbColor {
        self.into()
    }
}

impl From<(u8, u8, u8)> for RgbColor {
    fn from(value: (u8, u8, u8)) -> RgbColor {
        RgbColor(value.0, value.1, value.2)
    }
}

impl From<RgbColor> for (u8, u8, u8) {
    fn from(value: RgbColor) -> (u8, u8, u8) {
        (value.0, value.1, value.2)
    }
}

#[cfg(feature = "anstyle")]
#[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
impl From<RgbColor> for anstyle::RgbColor {
    /// Convert an `RgbColor` to an [`anstyle::RgbColor`]
    fn from(value: RgbColor) -> anstyle::RgbColor {
        anstyle::RgbColor(value.0, value.1, value.2)
    }
}
