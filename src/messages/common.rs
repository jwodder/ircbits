macro_rules! common_cow {
    ($t:ident, $err:ty) => {
        impl<'a> From<$t<'a>> for Cow<'a, str> {
            fn from(value: $t<'a>) -> Cow<'a, str> {
                value.0
            }
        }

        impl std::fmt::Display for $t<'_> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }

        impl PartialEq<str> for $t<'_> {
            fn eq(&self, other: &str) -> bool {
                self.0 == other
            }
        }

        impl<'a> PartialEq<&'a str> for $t<'_> {
            fn eq(&self, other: &&'a str) -> bool {
                &self.0 == other
            }
        }

        impl AsRef<str> for $t<'_> {
            fn as_ref(&self) -> &str {
                self.0.as_ref()
            }
        }

        impl std::ops::Deref for $t<'_> {
            type Target = str;

            fn deref(&self) -> &str {
                &*self.0
            }
        }

        impl<'a> TryFrom<&'a str> for $t<'a> {
            type Error = $err;

            fn try_from(s: &'a str) -> Result<$t<'a>, $err> {
                Cow::from(s).try_into()
            }
        }

        impl TryFrom<String> for $t<'static> {
            type Error = $err;

            fn try_from(s: String) -> Result<$t<'static>, $err> {
                Cow::from(s).try_into()
            }
        }

        impl std::str::FromStr for $t<'static> {
            type Err = $err;

            fn from_str(s: &str) -> Result<$t<'static>, $err> {
                Cow::from(String::from(s)).try_into()
            }
        }
    };
}
