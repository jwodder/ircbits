/// Individual formatting effects that can be applied to IRC text.
///
/// `Attribute` values can be combined with bitwise operators to produce
/// [`AttributeSet`]s.
#[derive(Clone, Copy, Debug, strum::EnumIter, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum Attribute {
    Bold = 1 << 0,
    Italic = 1 << 1,
    Underline = 1 << 2,
    Strikethrough = 1 << 3,
    Monospace = 1 << 4,
    Reverse = 1 << 5,
}

impl Attribute {
    const COUNT: u8 = 6;

    /// Returns an iterator over all [`Attribute`] variants
    pub fn iter() -> AttributeIter {
        // To avoid the need for users to import the trait
        <Attribute as strum::IntoEnumIterator>::iter()
    }
}

impl<A: Into<AttributeSet>> std::ops::BitAnd<A> for Attribute {
    type Output = AttributeSet;

    fn bitand(self, rhs: A) -> AttributeSet {
        AttributeSet((self as u8) & rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::BitOr<A> for Attribute {
    type Output = AttributeSet;

    fn bitor(self, rhs: A) -> AttributeSet {
        AttributeSet((self as u8) | rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::BitXor<A> for Attribute {
    type Output = AttributeSet;

    fn bitxor(self, rhs: A) -> AttributeSet {
        AttributeSet((self as u8) ^ rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::Sub<A> for Attribute {
    type Output = AttributeSet;

    fn sub(self, rhs: A) -> AttributeSet {
        AttributeSet((self as u8) & !rhs.into().0)
    }
}

impl std::ops::Not for Attribute {
    type Output = AttributeSet;

    fn not(self) -> AttributeSet {
        AttributeSet::ALL - self
    }
}

/// A set of [`Attribute`] values.
///
/// `AttributeSet` values can be combined with bitwise operators and can be
/// iterated over.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct AttributeSet(u8);

impl AttributeSet {
    /// A set containing no [`Attribute`]s
    pub const EMPTY: AttributeSet = AttributeSet(0);

    /// A set containing all [`Attribute`]s
    pub const ALL: AttributeSet = AttributeSet((1 << Attribute::COUNT) - 1);

    /// Return a new set containing no [`Attribute`]s
    pub fn new() -> AttributeSet {
        AttributeSet(0)
    }

    /// Test whether the set is empty
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Test whether the set contains all [`Attribute`]s
    pub fn is_all(self) -> bool {
        self == Self::ALL
    }

    /// Test whether the set contains the given [`Attribute`]
    pub fn contains(self, attr: Attribute) -> bool {
        self.0 & (attr as u8) != 0
    }
}

impl From<Attribute> for AttributeSet {
    fn from(value: Attribute) -> AttributeSet {
        AttributeSet(value as u8)
    }
}

impl IntoIterator for AttributeSet {
    type Item = Attribute;
    type IntoIter = AttributeSetIter;

    fn into_iter(self) -> AttributeSetIter {
        AttributeSetIter::new(self)
    }
}

impl FromIterator<Attribute> for AttributeSet {
    fn from_iter<I: IntoIterator<Item = Attribute>>(iter: I) -> Self {
        iter.into_iter()
            .fold(AttributeSet::new(), |set, attr| set | attr)
    }
}

impl Extend<Attribute> for AttributeSet {
    fn extend<I: IntoIterator<Item = Attribute>>(&mut self, iter: I) {
        for attr in iter {
            *self |= attr;
        }
    }
}

#[cfg(feature = "anstyle")]
#[cfg_attr(docsrs, doc(cfg(feature = "anstyle")))]
impl From<AttributeSet> for anstyle::Effects {
    /// Convert an `AttributeSet` to an [`anstyle::Effects`]
    ///
    /// # Data Loss
    ///
    /// The [`Attribute::Monospace`] attribute is discarded during conversion,
    /// as it has no `anstyle::Effects` equivalents.
    fn from(value: AttributeSet) -> anstyle::Effects {
        let mut efs = anstyle::Effects::new();
        for attr in value {
            match attr {
                Attribute::Bold => efs |= anstyle::Effects::BOLD,
                Attribute::Italic => efs |= anstyle::Effects::ITALIC,
                Attribute::Underline => efs |= anstyle::Effects::UNDERLINE,
                Attribute::Strikethrough => efs |= anstyle::Effects::STRIKETHROUGH,
                Attribute::Monospace => (),
                Attribute::Reverse => efs |= anstyle::Effects::INVERT,
            }
        }
        efs
    }
}

impl<A: Into<AttributeSet>> std::ops::BitAnd<A> for AttributeSet {
    type Output = AttributeSet;

    fn bitand(self, rhs: A) -> AttributeSet {
        AttributeSet(self.0 & rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::BitAndAssign<A> for AttributeSet {
    fn bitand_assign(&mut self, rhs: A) {
        self.0 &= rhs.into().0;
    }
}

impl<A: Into<AttributeSet>> std::ops::BitOr<A> for AttributeSet {
    type Output = AttributeSet;

    fn bitor(self, rhs: A) -> AttributeSet {
        AttributeSet(self.0 | rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::BitOrAssign<A> for AttributeSet {
    fn bitor_assign(&mut self, rhs: A) {
        self.0 |= rhs.into().0;
    }
}

impl<A: Into<AttributeSet>> std::ops::BitXor<A> for AttributeSet {
    type Output = AttributeSet;

    fn bitxor(self, rhs: A) -> AttributeSet {
        AttributeSet(self.0 ^ rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::BitXorAssign<A> for AttributeSet {
    fn bitxor_assign(&mut self, rhs: A) {
        self.0 ^= rhs.into().0;
    }
}

impl<A: Into<AttributeSet>> std::ops::Sub<A> for AttributeSet {
    type Output = AttributeSet;

    fn sub(self, rhs: A) -> AttributeSet {
        AttributeSet(self.0 & !rhs.into().0)
    }
}

impl<A: Into<AttributeSet>> std::ops::SubAssign<A> for AttributeSet {
    fn sub_assign(&mut self, rhs: A) {
        self.0 &= !rhs.into().0;
    }
}

impl std::ops::Not for AttributeSet {
    type Output = AttributeSet;

    fn not(self) -> AttributeSet {
        AttributeSet(!self.0 & ((1 << Attribute::COUNT) - 1))
    }
}

/// An iterator over the [`Attribute`]s in an [`AttributeSet`]
#[derive(Clone, Debug)]
pub struct AttributeSetIter {
    inner: AttributeIter,
    set: AttributeSet,
}

impl AttributeSetIter {
    fn new(set: AttributeSet) -> AttributeSetIter {
        AttributeSetIter {
            inner: Attribute::iter(),
            set,
        }
    }
}

impl Iterator for AttributeSetIter {
    type Item = Attribute;

    fn next(&mut self) -> Option<Attribute> {
        self.inner.by_ref().find(|&attr| self.set.contains(attr))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (0, self.inner.size_hint().1)
    }
}

impl DoubleEndedIterator for AttributeSetIter {
    fn next_back(&mut self) -> Option<Attribute> {
        self.inner.by_ref().rfind(|&attr| self.set.contains(attr))
    }
}

impl std::iter::FusedIterator for AttributeSetIter {}
